extern crate serial;

use std::io;
use std::fs;
use std::str;
use std::env;
use std::fs::File;
use std::ffi::OsString;
use serial::prelude::*;
use std::io::prelude::*;
use std::time::Duration;

const SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud9600,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};
const EOF_CHAR: [u8; 1] = [0x1C];

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let filename = &arguments[1];
    let port_name = OsString::from(&arguments[2]);
    let data = fs::read_to_string(filename).expect("Could not find file.");
    let mut output_file = File::create("serial_response ".to_owned() + &filename)
        .expect("Could not create output file.");
    let parser = "\r\n";
    let max_buffer_len: usize = 64;
    let parsed_data: Vec<&str> = parse_data(&data, &parser, &max_buffer_len);
    let mut port = serial::open(&port_name).expect("Failed to open port.");
    port.configure(&SETTINGS).expect("Failed to set port settings.");
    port.flush().expect("Failed to flush port buffer.");
    serial::core::SerialDevice::set_timeout(&mut port, Duration::new(60, 0))
        .expect("Failed to set timeout.");
    let response = serial_write_segments_read(&mut port, parsed_data, &"S".as_bytes(), &(EOF_CHAR[0] as i8)).unwrap();
    for val in response {
        output_file.write(&[val as u8]).expect("Failed to write to output file.");
    }
    
}

fn serial_write_segments_read<'a, T: io::Write + io::Read>(port: &'a mut T, data: Vec<&'a str>, end_write_byte: &'a[u8], eof_read_byte: &'a i8) -> 
    Result<Vec<i8>, io::Error> 
{
    let mut result_vector = Vec::new();
    for segment in data {
        port.write(segment.as_bytes())?;
        port.write(end_write_byte)?;
        for byte in read_until_eof_char(port, eof_read_byte)? {
            result_vector.push(byte);
        }
    }
    Ok(result_vector)
}

fn parse_data<'a>(data: &'a String, parse_string: &'a str, max: &'a usize) -> Vec<&'a str> {
    let return_vec: Vec<&str>;
    if data.len() < *max {
        return_vec = vec![data];
    }
    else {
        return_vec = data.split(parse_string).collect();
        for item in &return_vec {
            if item.len() > *max {  }
        }
    }
    return_vec
}

fn read_until_eof_char<'a, T: io::Read>(mut port: &'a mut T, eof_char: &'a i8) -> Result<Vec<i8>, io::Error> {
    let mut result_vector = Vec::new();
    loop {
        let byte = read_i8(&mut port).expect("Port read error.");
        if &byte == eof_char { break; }
        if byte != 0 { result_vector.push(byte); }
    }
    Ok(result_vector)
}

fn read_i8<T: io::Read>(file: &mut T) -> Result<i8, io::Error>
{
    let mut read_buffer = [0u8; 1];
    file.read(&mut read_buffer)?;
    Ok(read_buffer[0] as i8)
}
