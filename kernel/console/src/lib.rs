#![no_std]
#![feature(alloc)]

extern crate keycodes_ascii;
extern crate vga_buffer;
#[macro_use] extern crate alloc;
extern crate spin;
extern crate dfqueue;
#[macro_use] extern crate log;
extern crate window_manager;
extern crate spawn;

// temporary, should remove this once we fix crate system
extern crate console_types; 
use console_types::{ConsoleEvent, ConsoleOutputEvent};


use keycodes_ascii::{Keycode, KeyAction, KeyEvent};
use alloc::string::String;
use vga_buffer::{VgaBuffer, ColorCode, DisplayPosition};
use core::sync::atomic::Ordering;
use spin::{Once, Mutex};
use dfqueue::{DFQueue, DFQueueConsumer, DFQueueProducer};

lazy_static! {
    /// An instance of FrameBufferText for writing to the console.
    /// try!(CONSOLE_FRAME_TEXT_BUFFER.lock().write_str("Hello world!\n").map_err(|_| "error in FrameBuffer's write_str()")); 
    pub static ref CONSOLE_FRAME_TEXT_BUFFER: Mutex<FrameTextBuffer> = Mutex::new(FrameTextBuffer::new());
}


static PRINT_PRODUCER: Once<DFQueueProducer<ConsoleEvent>> = Once::new();

const FONT_COLOR:usize = 0x90ee90;

/// Queues up the given `String` to be printed out to the console.
pub fn print_to_console(s: String) -> Result<(), &'static str> {
    let output_event = ConsoleEvent::OutputEvent(ConsoleOutputEvent::new(s));
    try!(PRINT_PRODUCER.try().ok_or("Console print producer isn't yet initialized!")).enqueue(output_event);
    Ok(())
}


/// Initializes the console by spawning a new thread to handle all console events, and creates a new event queue. 
/// This event queue's consumer is given to that console thread, and a producer reference to that queue is returned. 
/// This allows other modules to push console events onto the queue. 
pub fn init() -> Result<DFQueueProducer<ConsoleEvent>, &'static str> {
    let console_dfq: DFQueue<ConsoleEvent> = DFQueue::new();
    let console_consumer = console_dfq.into_consumer();
    let returned_producer = console_consumer.obtain_producer();
    PRINT_PRODUCER.call_once(|| {
        console_consumer.obtain_producer()
    });

    info!("console::init() trying to spawn_kthread...");
    try!(spawn::spawn_kthread(main_loop, console_consumer, String::from("console_loop"), None));
    // vga_buffer::print_str("console::init(): successfully spawned kthread!\n").unwrap();
    info!("console::init(): successfully spawned kthread!");

    try!(print_to_console(String::from(WELCOME_STRING)));
    try!(print_to_console(String::from("Console says hello!\n")));
    Ok(returned_producer)
}



/// the main console event-handling loop, runs on its own thread. 
/// This is the only thread that is allowed to touch the vga buffer!
/// It's an infinite loop, but will return if forced to exit because of an error. 
fn main_loop(consumer: DFQueueConsumer<ConsoleEvent>) -> Result<(), &'static str> { // Option<usize> just a placeholder because kthread functions must have one Argument right now... :(
    use core::ops::Deref;

    loop { 
        let event = match consumer.peek() {
            Some(ev) => ev,
            _ => { continue; }
        };

        match event.deref() {
            &ConsoleEvent::ExitEvent => {
                use core::fmt::Write;
                try!(CONSOLE_FRAME_TEXT_BUFFER.lock().write_str("\nSmoothly exiting console main loop.\n").map_err(|_| "error in VgaBuffer's write_str()"));
                return Ok(()); 
            }
            &ConsoleEvent::InputEvent(ref input_event) => {
                try!(handle_key_event(input_event.key_event));
            }
            &ConsoleEvent::OutputEvent(ref output_event) => {
                CONSOLE_FRAME_TEXT_BUFFER.lock().write_string_with_color(&output_event.text, FONT_COLOR);
            }
        }

        event.mark_completed();
    }

}


fn handle_key_event(keyevent: KeyEvent) -> Result<(), &'static str> {

    // Ctrl+D or Ctrl+Alt+Del kills the OS
    if keyevent.modifiers.control && keyevent.keycode == Keycode::D || 
            keyevent.modifiers.control && keyevent.modifiers.alt && keyevent.keycode == Keycode::Delete {
        panic!("Ctrl+D or Ctrl+Alt+Del was pressed, abruptly (not cleanly) stopping the OS!"); //FIXME do this better, by signaling the main thread
    }


    // EVERYTHING BELOW HERE WILL ONLY OCCUR ON A KEY PRESS (not key release)
    if keyevent.action != KeyAction::Pressed {
        return Ok(()); 
    }

    if keyevent.modifiers.control && keyevent.keycode == Keycode::T {
        // use core::fmt::Write;
        // use core::ops::DerefMut;
        let s = format!("PIT_TICKS={}, RTC_TICKS={:?}, SPURIOUS={}, APIC={}", 
            ::interrupts::pit_clock::PIT_TICKS.load(Ordering::Relaxed), 
            rtc::get_rtc_ticks().ok(),
            unsafe{::interrupts::SPURIOUS_COUNT},
            ::interrupts::APIC_TIMER_TICKS.load(Ordering::Relaxed)
        );

        CONSOLE_VGA_BUFFER.lock().write_string_with_color(&s, ColorCode::default());
        
        // debug!("PIT_TICKS={}, RTC_TICKS={:?}, SPURIOUS={}, APIC={}", 
        //         ::interrupts::pit_clock::PIT_TICKS.load(Ordering::Relaxed), 
        //         rtc::get_rtc_ticks().ok(),
        //         unsafe{::interrupts::SPURIOUS_COUNT},
        //         ::interrupts::APIC_TIMER_TICKS.load(Ordering::Relaxed));
        return; 
    }


    // PUT ADDITIONAL KEYBOARD-TRIGGERED BEHAVIORS HERE


    // home, end, page up, page down, up arrow, down arrow for the console
    if keyevent.keycode == Keycode::Home {
        CONSOLE_FRAME_TEXT_BUFFER.lock().display(DisplayPosition::Start);
        return;
    }
    if keyevent.keycode == Keycode::End {
        CONSOLE_FRAME_TEXT_BUFFER.lock().display(DisplayPosition::End);
        return;
    }
    if keyevent.keycode == Keycode::PageUp {
        CONSOLE_FRAME_TEXT_BUFFER.lock().display(DisplayPosition::Up(20));
        return;
    }
    if keyevent.keycode == Keycode::PageDown {
        CONSOLE_FRAME_TEXT_BUFFER.lock().display(DisplayPosition::Down(20));
        return;
    }
    if keyevent.modifiers.control && keyevent.modifiers.shift && keyevent.keycode == Keycode::Up {
        CONSOLE_FRAME_TEXT_BUFFER.lock().display(DisplayPosition::Up(1));
        return;
    }
    if keyevent.modifiers.control && keyevent.modifiers.shift && keyevent.keycode == Keycode::Down {
        CONSOLE_FRAME_TEXT_BUFFER.lock().display(DisplayPosition::Down(1));
        return;
    }
    
    //Pass TAB event to window manager
    //Window manager consumes dir();ection key input
    match keyevent.keycode {
         Keycode::Tab => {
             //window_manager::set_time_start();
             loop{
                 window_manager::window_switch();
             }
         }
         Keycode::Left|Keycode::Right|Keycode::Up|Keycode::Down => {
            window_manager::put_key_code(keyevent.keycode).unwrap();
         }
         _ => {}
    }

    //Pass Delete event and direction key event to 3d drawer application
    /*match keyevent.keycode {
         Keycode::Tab|Keycode::Delete|Keycode::Left|Keycode::Right|Keycode::Up|Keycode::Down => {
            graph_drawer::put_key_code(keyevent.keycode).unwrap();
         }
         _ => {}
    }*/


    match keyevent.keycode.to_ascii(keyevent.modifiers) {
        Some(c) => { 
            // we echo key presses directly to the console without queuing an event
            // trace!("  {}  ", c);
            use alloc::string::ToString;
            CONSOLE_FRAME_TEXT_BUFFER.lock().write_string_with_color(&c.to_string(), FONT_COLOR);
        }
        // _ => { println!("Couldn't get ascii for keyevent {:?}", keyevent); } 
        _ => { } 
    }

    Ok(())
}




// this doesn't line up as shown here because of the escaped backslashes,
// but it lines up properly when printed :)
const WELCOME_STRING: &'static str = "\n\n
 _____ _                              
|_   _| |__   ___  ___  ___ _   _ ___ 
  | | | '_ \\ / _ \\/ __|/ _ \\ | | / __|
  | | | | | |  __/\\__ \\  __/ |_| \\__ \\
  |_| |_| |_|\\___||___/\\___|\\__,_|___/ \n\n";
