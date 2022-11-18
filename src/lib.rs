use std::{collections::HashMap, thread, time::Duration, sync::mpsc::Sender};
use std::result::Result::Ok;
use wmi::{WMIConnection, COMLibrary,Variant};
use anyhow::{self};
mod test;

#[derive(Debug, PartialEq)]
pub enum Measurement {
    Temperature(f64), 
    Memory(f64),
    TotalMemory(f64),
    CpuUtil(f64),
    FrameRate(f64),
    Network(String),
    Test(String),
    NaN,
}

// initialize the measurement thread. This creates a new thread that runs in the background and 
// send the measurements to 'tx', for each lap is sleeps for 'sleep_dur'
// last and most importantly, the bool 'assume' describes if we should initialize a new WMI connection, 
// or if we are going to assume that a connection is already established. The reason for this is because 
// many engines and framework already establish a connection and thus we might need to assume.

pub fn init_measurement_thread(tx: Sender<Measurement>, sleep_dur: Duration, assume: bool){

    thread::Builder::new().name("WMI Measurement Thread".to_string()).spawn(move || {

        let wmi: WMIConnection = match init_wmi_connection(assume) {
            Ok(wmi) => wmi,
            Err(_) => panic!("WMI failed"),
        };

        tx.send(get_total_memory(&wmi)).unwrap();

        loop {

            let results: Vec<Measurement> = vec![
                get_temp(&wmi),
                get_available_memory(&wmi),
                get_cpu_util(&wmi),
                // get_frame_rate(&wmi),
                temporary_get(&wmi),
            ];

            for res in results {
                if res != Measurement::NaN {
                    tx.send(res);
                }
            }

            thread::sleep(sleep_dur);
        }

    }).unwrap();

}


// initialize the WMI connection
// 'assume' decides if we are going to assume a wmi connection has already been made,
// or if we are gonna create a new one.
pub fn init_wmi_connection(assume: bool) -> Result<WMIConnection, anyhow::Error>{
    unsafe {
        let com_lib: COMLibrary;
        if assume {
            com_lib = COMLibrary::assume_initialized();
        } else {
            com_lib = COMLibrary::new().unwrap();
        }

        let wmi_con = WMIConnection::new(com_lib.into())?;

        Ok(wmi_con)
    }
}

// get the temperature of the machine, returns in Celsius.
// return 'Measurement::NaN' on error
pub fn get_temp(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT Temperature FROM Win32_PerfFormattedData_Counters_ThermalZoneInformation",
    )
    .unwrap();

    let data = match results.get(0) {
        Some(x) => x,
        _ => return Measurement::NaN,
    };

    let mut kelvin: f64 = 0.0;
    match data.get("Temperature") {
        Some(Variant::UI4(val)) => kelvin = *val as f64,
        _ => return Measurement::NaN,
    };
    
    Measurement::Temperature(kelvin - 273.0)
}

// returns cpu utilization
// return 'Measurement::NaN' on error
pub fn get_cpu_util(wmi: &WMIConnection) -> Measurement {
    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT LoadPercentage FROM Win32_Processor",
    )
    .unwrap();

    let data = match results.get(0) {
        Some(x) => x,
        _ => return Measurement::NaN,
    };

    let mut percent: f64 = 0.0;
    match data.get("LoadPercentage") {
        Some(Variant::UI2(val)) => percent = *val as f64,
        _ => return Measurement::NaN,
    };
    
    Measurement::CpuUtil(percent)
}

// get available memory (ram) returns the volume in bytes
// returns 'Measurement::NaN' on error
pub fn get_available_memory(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT AvailableBytes FROM Win32_PerfFormattedData_PerfOS_Memory",
    )
    .unwrap();

    let data = match results.get(0) {
        Some(x) => x,
        _ => return Measurement::NaN,
    };

    let mut bytes: f64 = 0.0; 
    match data.get("AvailableBytes") {
        Some(Variant::UI8(val)) => bytes = *val as f64,
        _ => return Measurement::NaN,
    };

    let kib = bytes / 1024.0;

    Measurement::Memory(kib)
}

// get the total amount of physical memory, returns in bytes
// return 'Measurement::NaN' on error
pub fn get_total_memory(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT TotalPhysicalMemory FROM Win32_ComputerSystem",
    )
    .unwrap();

    let data = match results.get(0) {
        Some(x) => x,
        _ => return Measurement::NaN,
    };

    let mut bytes: f64 = 0.0;
    
    match data.get("TotalPhysicalMemory") {
        Some(Variant::UI8(val)) => bytes = *val as f64,
        _ => return Measurement::NaN,
    };

    let kib = bytes / 1024.0;

    Measurement::TotalMemory(kib)
}



pub fn get_network_connection(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT Status FROM Win32_NetworkConnection",
    )
    .unwrap();

    let data = match results.get(0) {
        Some(x) => x,
        _ => return Measurement::NaN,
    };

    let mut status: String = "0.0".to_string();

    match data.get("Status") {
        Some(Variant::String(val)) => status = val.to_string(),
        _ => return Measurement::NaN,
    };

    Measurement::Network(status)
}



pub fn get_frame_rate(wmi: &WMIConnection) -> Measurement {
    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT CurrentRefreshRate FROM Win32_VideoController",
    )
    .unwrap();

    let data = match results.get(0) {
        Some(x) => x,
        _ => return Measurement::NaN,
    };

    let mut fr: f64 = 0.0;

    match data.get("CurrentRefreshRate") {
        Some(Variant::UI4(val)) => fr = *val as f64,
        _ => return Measurement::NaN,
    };

    Measurement::FrameRate(fr)
}

pub fn KiB_to_GiB(kib: f64) -> f64{
    kib / (1024.0 * 1024.0)
}


// temporary used to test different queries
fn temporary_get(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT Status FROM Win32_NetworkConnection",
    )
    .unwrap();

    let data = match results.get(0) {
        Some(x) => x,
        _ => return Measurement::NaN,
    };

    let mut bytes: String = "0.0".to_string();

    match data.get("Status") {
        Some(Variant::String(val)) => bytes = val.to_string(),
        _ => return Measurement::NaN,
    };

    Measurement::Test(bytes)
}