use std::error::Error;
use std::io::{stdin, stdout, Write};

use std::sync::Arc;
use std::sync::atomic::Ordering;

use midir::{Ignore, MidiInput, MidiInputConnection};

use crate::synth_state::SynthState;


pub fn initiate_midi_connection(synth_state: Arc<SynthState>) -> Result<MidiInputConnection<()>, Box<dyn Error>> {
  let mut input = String::new();

  let mut midi_in = MidiInput::new("midir reading input")?;
  midi_in.ignore(Ignore::None);

  // Get an input port (read from console if multiple are available)
  let in_ports = midi_in.ports();
  let in_port = match in_ports.len() {
    0 => return Err("no input port found".into()),
    1 => {
      println!(
        "Choosing the only available input port: {}",
        midi_in.port_name(&in_ports[0]).unwrap()
      );
      &in_ports[0]
    },
    _ => {
      println!("\nAvailable input ports:");
      for (i, p) in in_ports.iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p).unwrap());
      }
      print!("Please select input port: ");
      stdout().flush()?;
      let mut input = String::new();
      stdin().read_line(&mut input)?;
      in_ports
        .get(input.trim().parse::<usize>()?)
        .ok_or("invalid input port selected")?
    },
  };

  println!("\nOpening connection");
  let in_port_name = midi_in.port_name(in_port)?;

  // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
  let synth_state_clone = Arc::clone(&synth_state);
  let _conn_in = midi_in.connect(
    in_port,
    "midir-read-input",
    move |stamp, message, _| {
        if message.len() >= 3 {
                let status = message[0] & 0xF0;
                let note = message[1];
                let velocity = message[2];

                match status {
                    0x90 if velocity > 0 => { // Note On
                        synth_state_clone.last_key.store(note, Ordering::Relaxed);
                        synth_state_clone.has_key_pressed.store(true, Ordering::Relaxed);
                    }
                    0x80 | 0x90 => { // Note Off или Note On с vel=0
                        synth_state_clone.last_key.store(0, Ordering::Relaxed);
                        synth_state_clone.has_key_pressed.store(false, Ordering::Relaxed);
                    }
                    _ => {}
                }
            }
            
      println!("{}: {:?} (len = {})", stamp, message, message.len());
      let l = message[0];
      let _k: u8 = message[1]; //номер ноты!!!
      let _j = l == 144; //включена или выключена?!!!
      synth_state_clone.last_key.store(_k, Ordering::Relaxed);
      synth_state_clone.has_key_pressed.store(_j, Ordering::Relaxed);      
    },
    (),
  )?;

  println!(
    "Connection open, reading input from '{}' (press enter to exit) ...",
    in_port_name
  );

  Ok(_conn_in)
}