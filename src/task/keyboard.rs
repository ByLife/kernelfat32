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

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) { // pour les interruptions clavier
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream { 
    _private: (),
}

impl ScancodeStream { // impl pour ScancodeStream qui fait partie de futures_util
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream { // pour les interruptions clavier et les scancodes
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // fast path
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
        HandleControl::Ignore,
    );

    // buffer d'initialisation
    COMMAND_BUFFER.try_init_once(|| {
        Box::new(spin::Mutex::new(CommandBuffer::new()))
    }).expect("CommandBuffer already initialized");

    print!("> "); // prompt par défaut pour le shell

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => {
                        match character {
                            '\n' => COMMAND_BUFFER.try_get().unwrap().lock().execute(),
                            '\x08' => COMMAND_BUFFER.try_get().unwrap().lock().backspace(), // Backspace
                            c if c.is_ascii() => COMMAND_BUFFER.try_get().unwrap().lock().add_char(c),
                            _ => (), // ignorer les caractères non-ASCII
                        }
                    },
                    DecodedKey::RawKey(_) => (), // Ignorer les touches spéciales
                }
            }
        }
    }
}