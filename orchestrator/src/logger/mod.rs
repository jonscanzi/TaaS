macro_rules! println_with_time {
  
    () => { println!(); };
    ($($arg:tt)*) => {
        println!("{} ~ {}", chrono::Local::now().format("%H:%M:%S"), format!($($arg)*))
    }
}