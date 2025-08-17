// src/types/wled.ts

export interface LedsInfo {
  count: number;
}

export interface MapInfo {
  id: number;
}

export interface WledDevice {
  ip_address: string;
  port: number;
  name: string;
  version: string;
  leds: LedsInfo;
  udp_port: number;
  architecture: string;
  maps: MapInfo[];
}