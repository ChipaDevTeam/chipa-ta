import test from 'ava'
import { Indicators } from '../index.js'

test('RSI indicator', (t) => {
  const rsi = Indicators.rsi(14)
  t.is(typeof rsi.next(100), 'number')
})

test('SMA indicator', (t) => {
  const sma = Indicators.sma(10)
  t.is(typeof sma.next(100), 'number')
})

test('EMA indicator', (t) => {
  const ema = Indicators.ema(12)
  t.is(typeof ema.next(100), 'number')
})

test('MACD indicator', (t) => {
  const macd = Indicators.macd(12, 26, 9)
  const result = macd.next(100)
  t.true(typeof result === 'object')
})

test('Batch processing', (t) => {
  const sma = Indicators.sma(5)
  const inputs = [1, 2, 3, 4, 5]
  const results = sma.nextBatched(inputs)
  t.true(Array.isArray(results))
  t.is(results.length, inputs.length)
})

test('Indicator serialization', (t) => {
  const original = Indicators.rsi(14)
  const json = original.toJson()
  const restored = Indicators.fromString(json)
  t.is(typeof restored.next(100), 'number')
})

test('Indicator deserialization', (t) => {
  const json = '{"type":"Rsi","period":14}'
  const restored = Indicators.fromString(JSON.parse(json))
  t.is(typeof restored.next(100), 'number')
})
