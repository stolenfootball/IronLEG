use super::instruction::Operation;
use super::registers::Register;


const NUM_STATIONS: usize = 1024;

#[derive(Default)]
pub struct ReservationStation {
    pub station_number: usize,
    pub operation: Option<Operation>,
    pub argument_1: Option<Register>,
    pub argument_2: Option<Register>,
    pub immediate: Option<usize>,
    pub address: Option<usize>,
    pub destination: Option<usize>,
}

impl ReservationStation {
    pub fn new(station_number: usize) -> ReservationStation {
        ReservationStation {
            station_number,
            ..Default::default()
        }
    }
}

pub struct ReservationStations {
    stations: Vec<ReservationStation>,
}

impl Default for ReservationStations {
    fn default() -> Self {
        ReservationStations {
            stations: (0..NUM_STATIONS).map(|i| ReservationStation::new(i)).collect()
        }
    }
}

impl ReservationStations {
    pub fn get_free_station(&mut self) -> Option<&mut ReservationStation> {
        self.stations.iter_mut().find(|x| x.operation.is_none())
    }
}