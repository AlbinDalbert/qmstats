use std::{collections::HashMap, thread, time::{self, Duration}, sync::mpsc::{Sender, SyncSender}};
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
    NaN,
}

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
            ];

            for res in results {
                if res != Measurement::NaN {
                    tx.send(res).unwrap();
                }
            }

            thread::sleep(sleep_dur);
        }

    }).unwrap();

}


// initialize the WMI connection
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
pub fn get_temp(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT Temperature FROM Win32_PerfFormattedData_Counters_ThermalZoneInformation",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let kelvin: f64 = match data.get("Temperature").unwrap() {
        Variant::UI4(val) => *val as f64,
        _ => -1.0,
    };
    
    Measurement::Temperature(kelvin - 273.0)
}

// returns cpu utilization
// return -1 on error
pub fn get_cpu_util(wmi: &WMIConnection) -> Measurement {
    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT PercentProcessorTime FROM Win32_PerfFormattedData_PerfOS_Processor",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let percent: f64 = match data.get("PercentProcessorTime").unwrap() {
        Variant::UI8(val) => *val as f64,
        _ => -1.0,
    };
    
    Measurement::CpuUtil(percent)
}

// get available memory (ram) returns the volume in bytes

pub fn get_available_memory(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT AvailableBytes FROM Win32_PerfFormattedData_PerfOS_Memory",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let bytes: f64 = match data.get("AvailableBytes").unwrap() {
        Variant::UI8(val) => *val as f64,
        _ => -1.0,
    };

    let kib = bytes / 1024.0;

    Measurement::Memory(kib)
}

// get the total amount of physical memory, returns in bytes

pub fn get_total_memory(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = wmi
    .raw_query(
        "SELECT TotalPhysicalMemory FROM Win32_ComputerSystem",
    )
    .unwrap();

    let data = results.get(0).unwrap();

    let bytes: f64 = match data.get("TotalPhysicalMemory").unwrap() {
        Variant::UI8(val) => *val as f64,
        _ => 0.0,
    };

    Measurement::TotalMemory(bytes)
}