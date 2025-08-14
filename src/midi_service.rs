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

                let mut knopki = synth_state_clone.nazatie_knopki.lock().unwrap();

                match status {
                    0xB0 => {
                        if note==44 {
                          synth_state_clone.gate_attack.store(velocity, Ordering::Relaxed);
                        }
                        else if note==45{
                          synth_state_clone.gate_decay.store(velocity, Ordering::Relaxed);
                        }
                        else if note==46{
                          synth_state_clone.gate_release.store(velocity, Ordering::Relaxed);
                        }
                        else if note==47{
                          synth_state_clone.gate_sustain.store(velocity, Ordering::Relaxed);
                        }
                        else if note==35{
                          synth_state_clone.lpf_cutoff.store(velocity, Ordering::Relaxed);
                        }
                        else if note==34{
                          synth_state_clone.lpf_res_factor.store(velocity, Ordering::Relaxed);
                        }
                        else if note==36{
                          synth_state_clone.delay_mix.store(velocity, Ordering::Relaxed);
                        }
                        else if note==37{
                          synth_state_clone.delay_feed_back.store(velocity, Ordering::Relaxed);
                        }
                        else if note==38{
                          synth_state_clone.delay_delay_time.store(velocity, Ordering::Relaxed);
                        }
                        else if note==39{
                          synth_state_clone.reverb_decay_time.store(velocity, Ordering::Relaxed);
                        }
                        else if note==40{
                          synth_state_clone.reverb_dry_wet_mix.store(velocity, Ordering::Relaxed);
                        }
                        else if note==41{
                          synth_state_clone.glide_time.store(velocity, Ordering::Relaxed);
                        }
                        
                    }
                    0x90 if velocity > 0 => { // Note On
                      if let Some(i) = knopki.iter().position(|&nomer_nazato_knopki| nomer_nazato_knopki == note) {
                        let nomer_nazato_knopki = knopki.remove(i);
                        knopki.push(nomer_nazato_knopki);
                      }else{
                          knopki.push(note);
                        }
                        synth_state_clone.last_key.store(note, Ordering::Relaxed);
                        synth_state_clone.has_key_pressed.store(true, Ordering::Relaxed);
                    }
                    0x80 | 0x90 => { // Note Off или Note On с vel=0
                        knopki.retain(|&nomer_nazato_knopki| nomer_nazato_knopki != note);

                        if let Some(&last) = knopki.last() {
                        synth_state_clone.last_key.store(last, Ordering::Relaxed);
                        synth_state_clone.has_key_pressed.store(true, Ordering::Relaxed);
                    }else{ 
                      synth_state_clone.last_key.store(0, Ordering::Relaxed);
                      synth_state_clone.has_key_pressed.store(false, Ordering::Relaxed);
                    }
                }
                    _ => {}
                }
            }
            
      println!("{}: {:?} (len = {})", stamp, message, message.len());     
    },
    (),
  )?;

  println!(
    "Connection open, reading input from '{}' (press enter to exit) ...",
    in_port_name
  );


  println!("Closing connection");
  Ok(_conn_in)
}


