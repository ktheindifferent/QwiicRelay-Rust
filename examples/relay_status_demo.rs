use qwiic_relay_rs::RelayStatus;

fn main() {
    println!("RelayStatus Demo");
    println!("================");
    
    // Demonstrate enum values
    println!("RelayStatus::Off = {}", RelayStatus::Off as u8);
    println!("RelayStatus::On = {}", RelayStatus::On as u8);
    
    // Demonstrate bool conversions
    let off_status = RelayStatus::from(false);
    let on_status = RelayStatus::from(true);
    println!("\nBool to RelayStatus:");
    println!("false -> {:?}", off_status);
    println!("true -> {:?}", on_status);
    
    // Demonstrate RelayStatus to bool
    println!("\nRelayStatus to bool:");
    println!("RelayStatus::Off -> {}", bool::from(RelayStatus::Off));
    println!("RelayStatus::On -> {}", bool::from(RelayStatus::On));
    
    // Demonstrate u8 conversions
    println!("\nu8 to RelayStatus:");
    println!("0 -> {:?}", RelayStatus::from(0u8));
    println!("1 -> {:?}", RelayStatus::from(1u8));
    println!("255 -> {:?}", RelayStatus::from(255u8));
    
    // Demonstrate equality
    println!("\nEquality checks:");
    println!("RelayStatus::Off == RelayStatus::Off: {}", RelayStatus::Off == RelayStatus::Off);
    println!("RelayStatus::On == RelayStatus::On: {}", RelayStatus::On == RelayStatus::On);
    println!("RelayStatus::Off == RelayStatus::On: {}", RelayStatus::Off == RelayStatus::On);
}