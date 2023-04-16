import json
import struct
import time
from datetime import datetime
from typing import List, Optional

from bluepy.btle import (
    UUID,
    BTLEDisconnectError,
    Characteristic,
    DefaultDelegate,
    Peripheral,
    Scanner,
)
from icecream import ic


def c2f(celsius):
    return (celsius * 1.8) + 32


# ===============================
# Script guards for correct usage
# ===============================


# ====================================
# Utility functions for WavePlus class
# ====================================


def parseSerialNumber(ManuDataHexStr):
    if ManuDataHexStr is None or ManuDataHexStr == "None":
        SN = "Unknown"
    else:
        ManuData = bytearray.fromhex(ManuDataHexStr)

        if ((ManuData[1] << 8) | ManuData[0]) == 0x0334:
            SN = ManuData[2]
            SN |= ManuData[3] << 8
            SN |= ManuData[4] << 16
            SN |= ManuData[5] << 24
        else:
            SN = "Unknown"
    return SN


def get_peripheral(macAddr: str) -> Peripheral:
    print(f"MacAddr {macAddr}")
    periph = None
    while periph is None:
        try:
            periph = Peripheral(macAddr)
        except BTLEDisconnectError as e:
            print(f"BTLEDisconnectError {e}")
            print("get_peripheral: Retrying in 10 seconds")
            time.sleep(10)
    print(f"Peripheral {periph}")
    return periph


def get_characteristics(periph: Peripheral, uuid: UUID) -> Characteristic:
    charectoristics = periph.getCharacteristics(uuid=uuid)
    for char in charectoristics:
        print(f"char {char}")

    curr_val_char = charectoristics[0]
    return curr_val_char


def scan_for_sn(serial_number) -> Optional[str]:
    scanner = Scanner().withDelegate(DefaultDelegate())
    devices = scanner.scan(10)  # 0.1 seconds scan period
    print(f"devices {len(devices)}")
    for dev in devices:
        ManuData = dev.getValueText(255)
        print(f"{dev} {ManuData}")
        serial_number = parseSerialNumber(ManuData)
        if serial_number == serial_number:
            print(f"matched {dev.addr}")
            return dev.addr
        else:
            print(f"not matched {dev.addr}")
            return None


def disconnect(periph):
    periph.disconnect()


def conv2radon(radon_raw):
    radon = -1
    if 0 <= radon_raw <= 16383:
        radon = radon_raw
    return radon


def read(curr_val_char):
    rawdata = curr_val_char.read()
    rawdata = struct.unpack("<BBBBHHHHHHHH", rawdata)
    sensors = Sensors()
    sensors.set(rawdata)
    return sensors


sensor_units = [
    "%rH",
    "Bq/m3",
    "Bq/m3",
    "degF",
    "hPa",
    "ppm",
    "ppb",
    "time",
]


NUMBER_OF_SENSORS = 7
SENSOR_IDX_HUMIDITY = 0
SENSOR_IDX_RADON_SHORT_TERM_AVG = 1
SENSOR_IDX_RADON_LONG_TERM_AVG = 2
SENSOR_IDX_TEMPERATURE = 3
SENSOR_IDX_REL_ATM_PRESSURE = 4
SENSOR_IDX_CO2_LVL = 5
SENSOR_IDX_VOC_LVL = 6


class Sensors:
    def __init__(self):
        self.sensor_version = None
        self.sensor_data = [None] * NUMBER_OF_SENSORS

    def set(self, rawData):
        self.sensor_version = rawData[0]
        if self.sensor_version == 1:
            self.sensor_data[SENSOR_IDX_HUMIDITY] = rawData[1] / 2.0
            self.sensor_data[SENSOR_IDX_RADON_SHORT_TERM_AVG] = conv2radon(rawData[4])
            self.sensor_data[SENSOR_IDX_RADON_LONG_TERM_AVG] = conv2radon(rawData[5])
            self.sensor_data[SENSOR_IDX_TEMPERATURE] = rawData[6] / 100.0
            self.sensor_data[SENSOR_IDX_REL_ATM_PRESSURE] = rawData[7] / 50.0
            self.sensor_data[SENSOR_IDX_CO2_LVL] = rawData[8] * 1.0
            self.sensor_data[SENSOR_IDX_VOC_LVL] = rawData[9] * 1.0
        else:
            raise Exception(f"Unknown sensor version {self.sensor_version}")

    def getValue(self, sensor_index):
        return self.sensor_data[sensor_index]

    def getUnit(self, sensor_index):
        return sensor_units[sensor_index]


def main():
    serialNumber = 2930062999
    samplePeriod = None

    print("Device serial number: %s" % serialNumber)
    MacAddr = "58:93:d8:8b:12:2a"

    perif: Peripheral = get_peripheral(MacAddr)
    uuid = UUID("b42e2a68-ade7-11e4-89d3-123b93f75cba")
    cvc: Characteristic = get_characteristics(perif, uuid)
    while True:
        print(f"cvc type{type(cvc)} {cvc}")
        # read values
        sensors = read(cvc)
        handle_sensor_values(sensors)
        if samplePeriod is None:
            break
        print(f"sleeping for {samplePeriod} seconds")
        time.sleep(samplePeriod)


def handle_sensor_values(sensors):
    humidity = f"{sensors.getValue(SENSOR_IDX_HUMIDITY)} {sensors.getUnit(SENSOR_IDX_HUMIDITY)}"
    radon_st_avg = f"{sensors.getValue(SENSOR_IDX_RADON_SHORT_TERM_AVG)} {sensors.getUnit(SENSOR_IDX_RADON_SHORT_TERM_AVG)}"
    radon_lt_avg = f"{sensors.getValue(SENSOR_IDX_RADON_LONG_TERM_AVG)} {sensors.getUnit(SENSOR_IDX_RADON_LONG_TERM_AVG)}"
    temperature = f"{c2f(sensors.getValue(SENSOR_IDX_TEMPERATURE))} {sensors.getUnit(SENSOR_IDX_TEMPERATURE)}"
    pressure = f"{sensors.getValue(SENSOR_IDX_REL_ATM_PRESSURE)} {sensors.getUnit(SENSOR_IDX_REL_ATM_PRESSURE)}"
    CO2_lvl = (
        f"{sensors.getValue(SENSOR_IDX_CO2_LVL)} {sensors.getUnit(SENSOR_IDX_CO2_LVL)}"
    )
    VOC_lvl = (
        f"{sensors.getValue(SENSOR_IDX_VOC_LVL)} {sensors.getUnit(SENSOR_IDX_VOC_LVL)}"
    )
    curtime = datetime.now().strftime("%H:%M:%S")
    # Print data
    header = [
        "date",
        "Humidity",
        "Radon ST avg",
        "Radon LT avg",
        "Temperature",
        "Pressure",
        "CO2 level",
        "VOC level",
        "time",
    ]

    print(header)

    reading_date = now = datetime.now().strftime("%Y-%m-%d %H:%M")
    data = [
        reading_date,
        humidity,
        radon_st_avg,
        radon_lt_avg,
        temperature,
        pressure,
        CO2_lvl,
        VOC_lvl,
        curtime,
    ]

    for h, d in zip(header, data, strict=True):
        ic(f"{h}: {d}")
    save_to_json("waveplus.json", header, data)


def save_to_json(filename: str, header: List[str], data: List[str]) -> None:
    with open(filename, "w") as f:
        json.dump(dict(zip(header, data, strict=True)), f)


if __name__ == "__main__":
    main()