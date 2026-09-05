use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::ledc::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::units::*;

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();

    println!("Configuring output channel");

    let peripherals = Peripherals::take()?;

    let mut channel = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &config::TimerConfig::new().frequency(50.Hz().into()),  // サーボパルスの周期20ms
        )?,
        peripherals.pins.gpio27,
    )?;

    println!("Starting duty-cycle loop");

    //let max_duty = channel.get_max_duty();
    let max_duty = 32;     // get_max_duty()が256(=20ms)のため、
                                // サーボのパルス幅の範囲（0.5ms ～ 2.5ms）で
                                // 動かすためにmax_dutyを32(=2.5ms)とする
    println!("Max duty {max_duty}");
    for numerator in [1, 2, 3, 4, 5].iter().cycle() {
        println!("Duty {numerator}/5");
        channel.set_duty(max_duty * numerator / 5)?;
        FreeRtos::delay_ms(2000);
    }

    loop {
        FreeRtos::delay_ms(1000);
    }
}
