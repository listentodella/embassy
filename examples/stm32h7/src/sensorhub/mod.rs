#![allow(dead_code)]

use defmt::*;
use heapless::{Entry, FnvIndexMap, String, Vec};

#[derive(Debug, Clone)]
pub enum SensorType {
    Accelermeter,
    Gyroscope,
    Magnetic,
    Temperature,
    AmbientLight,
    Proximity,
}

enum Sensor {
    Accel(),
    Gyro(),
    Magnet(),
    Temperature(),
}

/// sensor的属性
/// 考虑到灵活性,属性名皆为`String`类型
/// 属性值类型则不固定
/// 不过考虑到`f32` 或 `f64` 在`sensor core`里除了提高实现难度外
/// 并没有其他明显的优势,而整型则可以简化逻辑
#[derive(Debug, Clone)]
pub enum Attr {
    _Uid(u64),
    Hwid(u8),
    Type(SensorType),
    SensorName(String<32>),
    VendorName(String<32>),
    Rates(Vec<u16, 8>),
    Ranges(Vec<(i32, i32), 4>),
    _Bias(Vec<f32, 3>),
}

/// 虚拟传感器,指没有物理器件的传感器
/// 它们的数据来源往往是一个或多个物理或虚拟传感器
/// `idx`: 为该sensor隶属于的`SensorType`的序号
/// `sensor_name`: 该sensor的名称,一般为型号名称
/// `vendor_name`: 该sensor的供应商名称
/// `sensor_type`: 该sensor的`SensorType`
/// `rate`: 该sensor支持的odr,
/// `listeners`: 该sensor的监听者请求的odr,并且记录了对应的odr的现存的请求者的数量
#[derive(Debug)]
pub struct VirtualSensor {
    idx: u8,
    sensor_name: String<32>,
    vendor_name: String<32>,
    sensor_type: SensorType,
    // rate: HashMap<f32, u32>,
    //TODO: 考虑将来把第二个元素设置为原子类型
    listeners: FnvIndexMap<u32, u32, 8>,
    attr: FnvIndexMap<String<8>, Attr, 8>,
}

/// 物理传感器,指有物理器件的传感器
/// `idx`: 为该sensor隶属于的`SensorType`的序号
/// `sensor_name`: 该senso的名称,一般为型号名称
/// `vendor_name`: 该sensor的供应商名称
/// `sensor_type`: 该sensor的`SensorType`
/// `rate`: 该sensor支持的odr,
/// `listeners`: 该sensor的监听者请求的odr,并且记录了对应的odr的现存的请求者的数量
#[derive(Debug)]
pub struct PhysicalSensor {
    idx: u8,
    sensor_name: String<32>,
    vendor_name: String<32>,
    sensor_type: SensorType,
    //TODO: 考虑将来把第二个元素设置为原子类型
    listeners: FnvIndexMap<u16, u16, 8>,
    attr: FnvIndexMap<String<32>, Attr, 8>,
}

pub trait SensorOps {
    fn open(&mut self, req_odr: u32) {
        info!("default open: {}", req_odr);
    }

    fn hw_open(&mut self) {
        warn!("forget to impl or not?");
    }

    fn close(&mut self) {
        info!("default close");
    }
    fn flush(&mut self) {
        info!("default flush");
    }
    fn batch(&mut self) {
        info!("default batch");
    }
}

impl PhysicalSensor {
    pub fn new(sensor_type: SensorType, idx: u8, sensor_name: String<32>, vendor_name: String<32>) -> PhysicalSensor {
        PhysicalSensor {
            idx,
            sensor_name,
            vendor_name,
            sensor_type,
            listeners: FnvIndexMap::new(),
            attr: FnvIndexMap::new(),
        }
    }

    pub fn publish_default_attributes(&mut self) {
        info!("publish default attrs");
        let attr_name = String::try_from("sensor_name").unwrap();
        if let Entry::Vacant(v) = self.attr.entry(attr_name) {
            dbg!("insert sensor name");
            v.insert(Attr::SensorName(self.sensor_name.clone())).unwrap();
        }

        let attr_name = String::try_from("vendor_name").unwrap();
        if let Entry::Vacant(v) = self.attr.entry(attr_name) {
            dbg!("insert vendor name");
            v.insert(Attr::VendorName(self.vendor_name.clone())).unwrap();
        }

        let attr_name = String::try_from("hw_idx").unwrap();
        if let Entry::Vacant(v) = self.attr.entry(attr_name) {
            dbg!("insert hw idx");
            v.insert(Attr::Hwid(self.idx)).unwrap();
        }

        let attr_name = String::try_from("sensor_type").unwrap();
        if let Entry::Vacant(v) = self.attr.entry(attr_name) {
            dbg!("insert sensor type");
            v.insert(Attr::Type(self.sensor_type.clone())).unwrap();
        }
    }

    pub fn update_attributes(&mut self, attr_name: String<32>, attr_value: Attr) {
        match self.attr.entry(attr_name) {
            Entry::Vacant(v) => {
                v.insert(attr_value.clone()).unwrap();
            }
            Entry::Occupied(o) => {
                o.insert(attr_value.clone());
            }
        }
    }
}

impl SensorOps for PhysicalSensor {
    fn open(&mut self, req_odr: u32) {
        info!("PhysicalSensor {}-{} open", self.sensor_name, self.idx);
        //self.open(_req_odr);
        let mut best_match = 0u16;
        let attr_name = String::try_from("rates").unwrap();
        let supported_rates = self.attr.get(&attr_name);

        if let Some(Attr::Rates(rates)) = supported_rates {
            info!("{} supported rates:{}", self.sensor_name, rates);
            best_match = find_closet_ge(rates, req_odr as u16).unwrap();
        }

        match self.listeners.entry(best_match) {
            Entry::Vacant(v) => {
                v.insert(1).unwrap();
            }
            Entry::Occupied(mut o) => {
                let cnt = *o.get_mut();
                o.insert(cnt + 1);
            }
        }

        info!("get best match odr = {} ", best_match);
        self.hw_open();
    }
}

/// find the closet item greeter than the target from the array
fn find_closet_ge<T>(array: &[T], a: T) -> Option<T>
where
    T: Copy + Ord,
{
    array.iter().filter(|&x| x >= &a).min().copied()
}
