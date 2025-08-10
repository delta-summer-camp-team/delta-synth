use std::error::Error;
use std::io::{stdin, stdout, Write};
use midir::{MidiOutput, MidiOutputConnection};

/// Opens a MIDI output port chosen by the user and returns the connection.
pub fn open_midi_output() -> Result<MidiOutputConnection, Box<dyn Error>> {
  let mut input = String::new();
  let midi_out = MidiOutput::new("midir send output")?;

  let out_ports = midi_out.ports();
  let out_port = match out_ports.len() {
    0 => return Err("no output port found".into()),
    1 => {
      println!(
        "Choosing the only available output port: {}",
        midi_out.port_name(&out_ports[0])?
      );
      &out_ports[0]
    }
    _ => {
      println!("\nAvailable output ports:");
      for (i, p) in out_ports.iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p)?);
      }
      print!("Please select output port: ");
      stdout().flush()?;
      input.clear();
      stdin().read_line(&mut input)?;
      out_ports
        .get(input.trim().parse::<usize>()?)
        .ok_or("invalid output port selected")?
    }
  };

  println!("\nOpening connection to '{}'", midi_out.port_name(out_port)?);
  let conn_out = midi_out.connect(out_port, "midir-write-output")?;
  Ok(conn_out)
}