
#[cfg(test)]
mod tests {
    use crate::*;

    // executing init_wmi_connection() multiple times on the same thread causes the program to crash.
    // For this reason running all tests at once will not work. testing_all() should be used instead.

    #[test]
    fn establish_wmi_connection() {

        let handle = thread::spawn(|| {
        
            match init_wmi_connection() {
                Ok(_) => assert!(true),
                Err(_) => assert!(false),
            };

        });
        match handle.join() {
            Ok(_) => assert!(true),
            Err(_) => assert!(false)
        };
        
    }

    #[test]
    fn check_temp() {

        let handle = thread::spawn(|| {
            
            let wmi = match init_wmi_connection() {
                Ok(wmi) => wmi,
                Err(_) => panic!("WMI failed"),
            };
            
            get_temp(&wmi);

        });
        match handle.join() {
            Ok(_) => assert!(true),
            Err(_) => assert!(false)
        };
    }

    #[test]
    fn check_cpu_util() {

        let handle = thread::spawn(|| {
            
            let wmi = match init_wmi_connection() {
                Ok(wmi) => wmi,
                Err(_) => panic!("WMI failed"),
            };
            
            get_cpu_util(&wmi);

        });
        match handle.join() {
            Ok(_) => assert!(true),
            Err(_) => assert!(false)
        };
    }

    #[test]
    fn check_available_memory() {

        let handle = thread::spawn(|| {
            
            let wmi = match init_wmi_connection() {
                Ok(wmi) => wmi,
                Err(_) => panic!("WMI failed"),
            };
            
            get_available_memory(&wmi);
            
        });
        match handle.join() {
            Ok(_) => assert!(true),
            Err(_) => assert!(false)
        };
    }

}