use std::{
    env,
    thread::{self, JoinHandle},
};

use anyhow::{Context, Result, anyhow};
use greetd_ipc::codec::TokioCodec;
use tokio::{
    net::UnixStream,
    runtime::{self},
    sync::mpsc::{self, Receiver, Sender},
};

pub enum Command {
    Quit,
    Entered(String),
    Next,
    Session(String),
}

#[derive(Clone, Copy)]
pub enum InputType {
    None,
    Password,
    Visible,
}

pub enum Responce {
    Success, // should close
    Error(String),
    Message(String),
    GetInput(InputType),
    GetSession,
}

pub struct Handle {
    pub send: mpsc::Sender<Command>,
    pub recieve: mpsc::Receiver<Responce>,
    pub join: JoinHandle<()>,
}

impl Handle {
    pub fn make_handle<F>(user: String, f: F) -> Result<()>
    where
        F: FnOnce(&mut Handle) -> Result<()>,
    {
        let mut handle = run(user).with_context(|| "while starting background thread")?;

        match f(&mut handle) {
            Ok(()) => {}
            Err(err) => {
                handle.close()?;
                return Err(err);
            }
        };

        handle.close()?;

        Ok(())
    }

    fn close(mut self) -> Result<()> {
        let _ = self.send.blocking_send(Command::Quit);

        while self.recieve.blocking_recv().is_some() {}

        self.join.join().unwrap();

        Ok(())
    }

    pub fn send_command(&mut self, command: Command) -> Result<()> {
        self.send.blocking_send(command)?;

        Ok(())
    }

    pub fn get_response(&mut self) -> Option<Responce> {
        self.recieve.try_recv().ok()
    }
}

fn run(user: String) -> Result<Handle> {
    let (otx, mut trx) = mpsc::channel(4);
    let (ttx, orx) = mpsc::channel(4);

    let join = thread::spawn(move || {
        let rt = match runtime::Builder::new_current_thread().enable_io().build() {
            Ok(rt) => rt,
            Err(err) => {
                ttx.blocking_send(Responce::Error(format!(
                    "failed to make tokio runtime with error: {err}"
                )))
                .unwrap();
                return;
            }
        };

        match rt.block_on(run_async(user, &mut trx, &ttx)) {
            Ok(()) => {}
            Err(err) => ttx
                .blocking_send(Responce::Error(format!("{err}")))
                .unwrap(),
        }
    });

    Ok(Handle {
        send: otx,
        recieve: orx,
        join,
    })
}

async fn run_authflow(
    user: String,
    commands: &mut Receiver<Command>,
    responce: &Sender<Responce>,
    s: &mut UnixStream,
) -> Result<bool> {
    let mut success = false;

    greetd_ipc::Request::CreateSession {
        username: user.to_string(),
    }
    .write_to(s)
    .await?;

    let mut current_type: Option<InputType> = None;

    loop {
        match greetd_ipc::Response::read_from(s).await? {
            greetd_ipc::Response::Success => {
                success = true;
                break;
            }
            greetd_ipc::Response::Error {
                error_type,
                description,
            } => {
                match error_type {
                    greetd_ipc::ErrorType::Error => {
                        responce.send(Responce::Error(description)).await?
                    }
                    greetd_ipc::ErrorType::AuthError => {
                        responce.send(Responce::Error(description)).await?
                    }
                }

                let socket = env::var("GREETD_SOCK").unwrap();

                *s = UnixStream::connect(socket).await?;

                greetd_ipc::Request::CreateSession {
                    username: user.to_string(),
                }
                .write_to(s)
                .await?;

                // greetd_ipc::Request::CancelSession.write_to(s).await?;
                // greetd_ipc::Request::CreateSession {
                //     username: user.clone(),
                // }
                // .write_to(s)
                // .await?;

                // if let Some(cur) = current_type {
                //     responce.send(Responce::GetInput(cur)).await?;

                //     let command = commands
                //         .recv()
                //         .await
                //         .with_context(|| "should get a responce".to_string())?;

                //     match command {
                //         Command::Quit => {
                //             break;
                //         }
                //         Command::Entered(str) => {
                //             greetd_ipc::Request::PostAuthMessageResponse {
                //                 response: Some(str),
                //             }
                //             .write_to(s)
                //             .await?;
                //         }
                //         Command::Session(_) => Err(anyhow!("don't need session yet"))?,
                //         Command::Next => Err(anyhow!("need a password"))?,
                //     }
                // }
            }
            greetd_ipc::Response::AuthMessage {
                auth_message_type,
                auth_message,
            } => {
                let resp = match auth_message_type {
                    greetd_ipc::AuthMessageType::Visible => {
                        responce.send(Responce::Message(auth_message)).await?;
                        responce
                            .send(Responce::GetInput(InputType::Visible))
                            .await?;

                        current_type = Some(InputType::Visible);

                        let command = commands
                            .recv()
                            .await
                            .with_context(|| "should get a responce".to_string())?;

                        match command {
                            Command::Quit => {
                                break;
                            }
                            Command::Entered(str) => Some(str),
                            Command::Session(_) => Err(anyhow!("don't need session yet"))?,
                            Command::Next => Err(anyhow!("need a password"))?,
                        }
                    }
                    greetd_ipc::AuthMessageType::Secret => {
                        responce.send(Responce::Message(auth_message)).await?;
                        responce
                            .send(Responce::GetInput(InputType::Password))
                            .await?;

                        current_type = Some(InputType::Password);

                        let command = commands
                            .recv()
                            .await
                            .with_context(|| "should get a responce".to_string())?;

                        match command {
                            Command::Quit => {
                                break;
                            }
                            Command::Entered(str) => Some(str),
                            Command::Session(_) => Err(anyhow!("don't need session yet"))?,
                            Command::Next => Err(anyhow!("need a password"))?,
                        }
                    }
                    greetd_ipc::AuthMessageType::Info => {
                        responce.send(Responce::Message(auth_message)).await?;
                        responce.send(Responce::GetInput(InputType::None)).await?;

                        current_type = Some(InputType::None);

                        let command = commands
                            .recv()
                            .await
                            .with_context(|| "should get a responce".to_string())?;

                        match command {
                            Command::Quit => {
                                break;
                            }
                            Command::Entered(_) => Err(anyhow!("need a password"))?,
                            Command::Session(_) => Err(anyhow!("don't need session yet"))?,
                            Command::Next => None,
                        }
                    }
                    greetd_ipc::AuthMessageType::Error => {
                        responce.send(Responce::Message(auth_message)).await?;
                        responce.send(Responce::GetInput(InputType::None)).await?;

                        current_type = Some(InputType::None);

                        let command = commands
                            .recv()
                            .await
                            .with_context(|| "should get a responce".to_string())?;

                        match command {
                            Command::Quit => {
                                break;
                            }
                            Command::Entered(_) => Err(anyhow!("need a password"))?,
                            Command::Session(_) => Err(anyhow!("don't need session yet"))?,
                            Command::Next => None,
                        }
                    }
                };

                greetd_ipc::Request::PostAuthMessageResponse { response: resp }
                    .write_to(s)
                    .await?;
            }
        }
    }

    Ok(success)
}

async fn run_async(
    user: String,
    commands: &mut Receiver<Command>,
    responce: &Sender<Responce>,
) -> Result<()> {
    let socket = env::var("GREETD_SOCK").unwrap();

    let mut s = UnixStream::connect(socket).await?;

    loop {
        if run_authflow(user.clone(), commands, responce, &mut s).await? {
            responce.send(Responce::GetSession).await?;

            let command = commands
                .recv()
                .await
                .with_context(|| "failed to get back session responce")?;

            let session = match command {
                Command::Quit => break,
                Command::Entered(_) => panic!("invalid state"),
                Command::Next => panic!("invalid state"),
                Command::Session(session) => session,
            };

            greetd_ipc::Request::StartSession {
                cmd: vec!["sh".to_string(), "-c".to_string(), session],
                env: vec![],
            }
            .write_to(&mut s)
            .await?;

            match greetd_ipc::Response::read_from(&mut s).await? {
                greetd_ipc::Response::Success => {
                    responce.send(Responce::Success).await?;
                    break;
                }

                greetd_ipc::Response::Error {
                    error_type,
                    description,
                } => {
                    match error_type {
                        greetd_ipc::ErrorType::Error => {
                            responce.send(Responce::Error(description)).await?
                        }
                        greetd_ipc::ErrorType::AuthError => {
                            responce
                                .send(Responce::Error("description".to_string()))
                                .await?
                        }
                    }
                    greetd_ipc::Request::CreateSession {
                        username: user.clone(),
                    }
                    .write_to(&mut s)
                    .await?
                }

                greetd_ipc::Response::AuthMessage { .. } => {
                    Err(anyhow!("got auth message when waiting for success/failure"))?
                }
            }
        } else {
            break;
        }
    }

    Ok(())
}
