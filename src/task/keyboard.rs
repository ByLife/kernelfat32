use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use alloc::boxed::Box;
use crate::commands::CommandBuffer;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();
static COMMAND_BUFFER: OnceCell<Box<spin::Mutex<CommandBuffer>>> = OnceCell::uninit();

pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("attention: queue pleine");
        } else {
            WAKER.wake();
        }
    } else {
        println!("attention: queue non initialisée");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new doit etre appelé une seule fois");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("queue non initialisée");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::MapLettersToUnicode
    );

    COMMAND_BUFFER.try_init_once(|| {
        Box::new(spin::Mutex::new(CommandBuffer::new()))
    }).expect("CommandBuffer déjà initialisé");

    print!("> ");

    let mut ctrl_pressed = false;

    while let Some(scancode) = scancodes.next().await {
        // check si c'est la touche CTRL (0x1D pour appuyé, 0x9D pour relâché)
        if scancode == 0x1D {
            ctrl_pressed = true;
            continue;
        } else if scancode == 0x9D {
            ctrl_pressed = false;
            continue;
        }

        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        if ctrl_pressed && character == 'c' {
                            println!("\ncommande interrompue");
                            print!("> ");
                            COMMAND_BUFFER.try_get().unwrap().lock().clear();
                            continue;
                        }

                        match character {
                            '\u{8}' | '\u{7f}' => {
                                COMMAND_BUFFER.try_get().unwrap().lock().backspace();
                            }
                            '\n' => {
                                COMMAND_BUFFER.try_get().unwrap().lock().execute();
                            }
                            c if c.is_ascii() && !c.is_control() => {
                                COMMAND_BUFFER.try_get().unwrap().lock().add_char(c);
                            }
                            _ => {}
                        }
                    }
                    DecodedKey::RawKey(_) => {}
                }
            }
        }
    }
}