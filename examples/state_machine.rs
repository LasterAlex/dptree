//! You can see image of the state machine at /img/state_machine.gif

extern crate dispatch_tree as dptree;

use dispatch_tree::{Handler, HandlerBuilder};
use std::fmt::{Display, Formatter};
use std::io::Write;

#[derive(Debug)]
pub enum CommandState {
    Active,
    Paused,
    Inactive,
    Exit,
}

impl Display for CommandState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            CommandState::Active => f.write_str("Active"),
            CommandState::Paused => f.write_str("Paused"),
            CommandState::Inactive => f.write_str("Inactive"),
            CommandState::Exit => f.write_str("Exit"),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Begin,
    Pause,
    Resume,
    End,
    Exit,
}

impl Event {
    fn parse(input: &str) -> Option<Self> {
        match input {
            "begin" => Some(Event::Begin),
            "pause" => Some(Event::Pause),
            "resume" => Some(Event::Resume),
            "end" => Some(Event::End),
            "exit" => Some(Event::Exit),
            _ => None,
        }
    }
}

mod transitions {
    use super::*;

    pub fn begin() -> impl Handler<(Event, CommandState), Res = CommandState> {
        dptree::filter(dptree::matches!((Event::Begin, _)))
            .end_point(|| async { CommandState::Active })
    }

    pub fn pause() -> impl Handler<(Event, CommandState), Res = CommandState> {
        dptree::filter(dptree::matches!((Event::Pause, _)))
            .end_point(|| async { CommandState::Paused })
    }

    pub fn end() -> impl Handler<(Event, CommandState), Res = CommandState> {
        dptree::filter(dptree::matches!((Event::End, _)))
            .end_point(|| async { CommandState::Inactive })
    }

    pub fn resume() -> impl Handler<(Event, CommandState), Res = CommandState> {
        dptree::filter(dptree::matches!((Event::Resume, _)))
            .end_point(|| async { CommandState::Active })
    }

    pub fn exit() -> impl Handler<(Event, CommandState), Res = CommandState> {
        dptree::filter(dptree::matches!((Event::Exit, _)))
            .end_point(|| async { CommandState::Exit })
    }
}

#[rustfmt::skip]
fn active_handler() -> impl Handler<(Event, CommandState), Res = CommandState> {
    dptree::filter(dptree::matches!((_, CommandState::Active)))
        .and_then(
            dptree::dispatch()
                .to(transitions::pause())
                .to(transitions::end())
                .build()
        )
}

#[rustfmt::skip]
fn paused_handler() -> impl Handler<(Event, CommandState), Res = CommandState> {
    dptree::filter(dptree::matches!((_, CommandState::Paused)))
        .and_then(
            dptree::dispatch()
                .to(transitions::resume())
                .to(transitions::end())
                .build()
        )
}

#[rustfmt::skip]
fn inactive_handler() -> impl Handler<(Event, CommandState), Res = CommandState> {
    dptree::filter(dptree::matches!((_, CommandState::Inactive)))
        .and_then(
            dptree::dispatch()
                .to(transitions::begin())
                .to(transitions::exit())
                .build()
        )
}

fn exit_handler() -> impl Handler<(Event, CommandState), Res = CommandState> {
    dptree::filter(dptree::matches!((_, CommandState::Exit))).and_then(dptree::dispatch().build())
}

#[tokio::main]
async fn main() {
    let mut state = CommandState::Inactive;

    let dispatcher = dptree::dispatch::<(Event, CommandState), CommandState>()
        .to(active_handler())
        .to(paused_handler())
        .to(inactive_handler())
        .to(exit_handler())
        .build();

    loop {
        println!("|| Current state is {}", state);
        print!(">> ");
        std::io::stdout().flush().unwrap();

        let mut cmd = String::new();
        std::io::stdin().read_line(&mut cmd).unwrap();

        let str = cmd.trim();
        let event = Event::parse(str);

        let new_state = match event {
            Some(event) => match dispatcher.handle((event, state)).await {
                Ok(state) => state,
                Err((_, the_state)) => {
                    println!("There is no transition for the event");
                    state = the_state;
                    continue;
                }
            },
            _ => {
                println!("Unknown event");
                continue;
            }
        };
        state = new_state;
    }
}
