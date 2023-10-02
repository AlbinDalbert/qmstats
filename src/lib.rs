use std::{collections::HashMap, thread, time::Duration, sync::mpsc::Sender};
use std::result::Result::Ok;
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use wmi::{WMIConnection, COMLibrary,Variant};
use anyhow::{self};
use nvml_wrapper::*;
mod test;

#[derive(Debug, PartialEq)]
pub enum Measurement {
    Temperature(f64), 
    Memory(f64),
    TotalMemory(f64),
    CpuUtil(f64),
    FrameRate(f64),
    Network(String),
    VramUsed(u64),
    VramTotal(u64),
    GpuUtil(u32),
    GpuTemp(u32),
    NaN,
}

// initialize the measurement thread. This creates a new thread that runs in the background and 
// send the measurements to 'tx', for each lap is sleeps for 'sleep_dur'
// last and most importantly, the bool 'assume' describes if we should initialize a new WMI connection, 
// or if we are going to assume that a connection is already established. The reason for this is because 
// many engines and framework already establish a connection and thus we might need to assume.

pub fn init_measurement_thread(tx: Sender<Measurement>, sleep_dur: Duration) {

    match thread::Builder::new().name("WMI Measurement Thread".to_string()).spawn(move || {

        let wmi: WMIConnection = match init_wmi_connection() {
            Ok(wmi) => wmi,
            Err(_) => panic!("WMI failed"),
        };

        let nvml = match Nvml::init() {
            Ok(x) => x,
            Err(_) => panic!("nvml broke"),
        };
    
        let device = match nvml.device_by_index(0) {
            Ok(x) => x,
            Err(_) => panic!("device broke"),
        };
        
        match tx.send(get_total_memory(&wmi)){
            Ok(()) => (),
            Err(x) => eprintln!("measurment error: {x:?}"),
        };

        match tx.send(get_total_vram(&device)){
            Ok(()) => (),
            Err(x) => eprintln!("measurment error: {x:?}"),
        };

        loop {

            let results: Vec<Measurement> = vec![
                get_temp(&wmi),
                get_available_memory(&wmi),
                get_cpu_util(&wmi),
                get_used_vram(&device),
                get_gpu_util(&device),
                get_gpu_temp(&device),
            ];

            for res in results {
                if res != Measurement::NaN {
                    match tx.send(res) {
                        Ok(()) => (),
                        Err(x) => eprintln!("measurment error: {x:?}"),
                    };
                }
            }
            
            thread::sleep(sleep_dur);
        }

    }) {
        Ok(_) => (),
        Err(e) => panic!("Thread failed: {}", e),
    };

}

// ------------- GPU DATA ------------------- //

pub fn get_used_vram(device: &Device) -> Measurement {
    let data = match device.memory_info(){
        Ok(x) => x,
        Err(_) => return Measurement::NaN,
    };

    let used = data.used;

    Measurement::VramUsed(used / 1024)
}

pub fn get_total_vram(device: &Device) -> Measurement {
    let data = match device.memory_info(){
        Ok(x) => x,
        Err(_) => return Measurement::NaN,
    };

    let used = data.total;

    Measurement::VramTotal(used / 1024)
}

pub fn get_gpu_util(device: &Device) -> Measurement {
    let data = match device.utilization_rates(){
        Ok(x) => x,
        Err(_) => return Measurement::NaN,
    };

    Measurement::GpuUtil(data.gpu)
}

pub fn get_gpu_temp(device: &Device) -> Measurement {
    let sensor = TemperatureSensor::Gpu;
    let data = match device.temperature(sensor){
        Ok(x) => x,
        Err(_) => return Measurement::NaN,
    };

    Measurement::GpuTemp(data)
}

// ------------- WMI DATA ------------------- //

// initialize the WMI connection
// 'assume' decides if we are going to assume a wmi connection has already been made,
// or if we are gonna create a new one.

pub fn init_wmi_connection() -> Result<WMIConnection, anyhow::Error>{
    unsafe {
        let com_lib = match COMLibrary::new() {
            Ok(x) => x,
            Err(_) => COMLibrary::assume_initialized(),
        };

        let wmi_con: WMIConnection = WMIConnection::new(com_lib.into())?;

        Ok(wmi_con)
    }
}

// get the temperature of the machine, returns in Celsius.
// return 'Measurement::NaN' on error
pub fn get_temp(wmi: &WMIConnection) -> Measurement {

    let results: Vec<HashMap<String, Variant>> = match wmi
    .raw_query(
        "SELECT Temperature FROM Win32_PerfFormattedData_Counters_ThermalZoneInformation",
    ) {
        Ok(x) => x,
        Err(_) => return Measurement::NaN,
    };

    let mut temp_total = 0.0;
    let mut count = 0.0;

    for hash in &results {
        temp_total += match hash.get("Temperature") {
            Some(Variant::UI4(val)) => *val as f64 - 273.0,
            _ => continue,
        };
        count+=1.0;
    }
    return Measurement::Temperature(temp_total/count);
}

// returns cpu utilization
// return 'Measurement::NaN' on error
pub fn get_cpu_util(wmi: &WMIConnection) -> Measurement {
    let results: Vec<HashMap<String, Variant>> = match wmi
    .raw_query(
        "SELECT LoadPercentage FROM Win32_Processor",
    ) {
        Ok(x) => x,
        Err(_) => return Measurement::NaN,
    };

    let mut util_total = 0.0;
    let mut count = 0.0;

    for hash in results {
        util_total += match hash.get("LoadPercentage") {
            Some(Variant::UI2(val)) => *val as f64,
            _ => continue,
        };
        count+=1.0;
    }

    return Measurement::CpuUtil(util_total/count);

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

    let bytes = match data.get("AvailableBytes") {
        Some(Variant::UI8(val)) => *val as f64,
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
    
    let bytes = match data.get("TotalPhysicalMemory") {
        Some(Variant::UI8(val)) => *val as f64,
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

    let status = match data.get("Status") {
        Some(Variant::String(val)) => val.to_string(),
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

    let fr = match data.get("CurrentRefreshRate") {
        Some(Variant::UI4(val)) => *val as f64,
        _ => return Measurement::NaN,
    };

    Measurement::FrameRate(fr)
}

#[allow(non_snake_case)]
pub fn KiB_to_GiB(kib: f64) -> f64{
    kib / (1024.0 * 1024.0)
}