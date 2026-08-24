use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::peripherals::Peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio27)?;

    loop {
        led.set_high()?;
        println!("Output High");
        FreeRtos::delay_ms(1000);

        led.set_low()?;
        println!("Output Low");
        FreeRtos::delay_ms(1000);
    }
}
