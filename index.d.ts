/* tslint:disable */
/* eslint-disable */

/* auto-generated by NAPI-RS */

export type CandleJs = Candle
export declare class Candle {
  price: number
  high: number
  low: number
  open: number
  close: number
  volume: number
  constructor(price: number, high: number, low: number, open: number, close: number, volume: number)
  static price(price: number): CandleJs
}
export type IndicatorJs = Indicators
export declare class Indicators {
  constructor()
  static fromString(json: unknown): IndicatorJs
  static ema(period: number): IndicatorJs
  static sma(period: number): IndicatorJs
  static rsi(period: number): IndicatorJs
  static macd(fastPeriod: number, slowPeriod: number, signalPeriod: number): IndicatorJs
  static tr(): IndicatorJs
  static atr(period: number): IndicatorJs
  static superTrend(multiplier: number, period: number): IndicatorJs
  toJson(): unknown
  next(input: number): unknown
  nextBatched(input: Array<number>): Array<unknown>
  nextCandle(candle: Candle): unknown
  nextCandles(candles: Array<Candle>): Array<unknown>
}
