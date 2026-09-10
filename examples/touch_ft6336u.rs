use std::{cell::RefCell, num::NonZeroU32};

use embedded_hal_bus::i2c::RefCellDevice;
use embedded_hal_compat::ReverseCompat;

use axp192::Axp192;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{InterruptType, PinDriver, Pull},
    i2c::{I2cConfig, I2cDriver},
    peripherals::Peripherals,
    task::notification::Notification,
    units::FromValueType,
};
use esp_idf_svc::systime::EspSystemTime;
use ft6x36::{Dimension, Ft6x36};

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().expect("Failed to take peripherals");
    let pins = peripherals.pins;

    // タッチコントローラー用の I2C0 (SDA=21, SCL=22)。
    let config = I2cConfig::new().baudrate(400_u32.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, pins.gpio21, pins.gpio22, &config)?;

    // 両ドライバーは main タスクでのみ使用し、I2C バスを共有する。
    let bus = RefCell::new(i2c);
    let mut axp = Axp192::new(RefCellDevice::new(&bus));
    m5sc2_init(&mut axp).unwrap();

    // ft6x36 0.4 は embedded-hal 0.2 の I2C trait を要求する。
    let touch_i2c = RefCellDevice::new(&bus).reverse();
    let mut touch = Ft6x36::new(touch_i2c, Dimension(320, 240));
    touch.init()?;
    log::info!("Touch controller: {:?}", touch.get_info());

    // GPIO より先に作成し、終了時は GPIO の購読が先に解除されるようにする。
    let notification = Notification::new();
    let notifier = notification.notifier();
    let mut touch_irq = PinDriver::input(pins.gpio39, Pull::Floating)?;
    touch_irq.set_interrupt_type(InterruptType::NegEdge)?;
    // SAFETY: ISR では ISR 対応の通知だけを行う。
    // 通知先の main タスクは GPIO の購読が解除されるまで存続する。
    unsafe {
        touch_irq.subscribe(move || {
            notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
        })?;
    }
    touch_irq.enable_interrupt()?;

    log::info!("Touch ready: touch, move, and release your finger.");
    loop {
        notification.wait_any();
        // HAL は通知のたびに割り込みを無効化するので、タスク側で再度有効化する。
        touch_irq.enable_interrupt()?;
        // I2C の読み取りとログ出力は ISR ではなく main タスクで行う。
        let time = EspSystemTime {}.now();
        match touch.get_touch_event() {
            Ok(raw) => {
                // p1/p2 には座標と Press/Contact などの状態が入る。
                log::info!("Raw touch: p1={:?}, p2={:?}", raw.p1, raw.p2);

                // 点がなくなったレポートも渡すことが重要。
                // ft6x36 0.4.0 はそれを操作の終了として判定する。
                if let Some(event) = touch.process_event(time, raw) {
                    log::info!("Recognized: {:?}", event);
                }
            }
            Err(error) => log::warn!("Touch read failed: {:?}", error),
        }
    }
}

// axp192-rsのexampleから持ってきた
fn m5sc2_init<I2C, E>(axp: &mut axp192::Axp192<I2C>) -> Result<(), E>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    // Default setup for M5Stack Core 2
    axp.set_dcdc1_voltage(3350)?; // Voltage to provide to the microcontroller (this one!)

    axp.set_ldo2_voltage(3300)?; // Peripherals (LCD, ...)
    axp.set_ldo2_on(true)?;

    axp.set_ldo3_voltage(2000)?; // Vibration motor
    axp.set_ldo3_on(false)?;

    axp.set_dcdc3_voltage(2800)?; // LCD backlight
    axp.set_dcdc3_on(true)?;

    axp.set_gpio1_mode(axp192::GpioMode12::NmosOpenDrainOutput)?; // Power LED
    axp.set_gpio1_output(false)?; // In open drain modes, state is opposite to what you might
                                  // expect

    axp.set_gpio2_mode(axp192::GpioMode12::NmosOpenDrainOutput)?; // Speaker
    axp.set_gpio2_output(true)?;

    axp.set_key_mode(
        // Configure how the power button press will work
        axp192::ShutdownDuration::Sd4s,
        axp192::PowerOkDelay::Delay64ms,
        true,
        axp192::LongPress::Lp1000ms,
        axp192::BootTime::Boot512ms,
    )?;

    axp.set_gpio4_mode(axp192::GpioMode34::NmosOpenDrainOutput)?; // LCD reset control

    axp.set_battery_voltage_adc_enable(true)?;
    axp.set_battery_current_adc_enable(true)?;
    axp.set_acin_current_adc_enable(true)?;
    axp.set_acin_voltage_adc_enable(true)?;

    // Actually reset the LCD
    axp.set_gpio4_output(false)?;
    axp.set_ldo3_on(true)?; // Buzz the vibration motor while intializing ¯\_(ツ)_/¯
                            //delay.delay_millis(100u32);
    FreeRtos::delay_ms(100u32); //_delay.delay_ms()ではダメだった
    axp.set_gpio4_output(true)?;
    axp.set_ldo3_on(false)?;
    //delay.delay_millis(100u32);
    FreeRtos::delay_ms(100u32);
    Ok(())
}
