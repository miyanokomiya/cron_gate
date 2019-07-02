import * as cron_gate from '../pkg/cron_gate'

const $form = document.getElementById('form')
const $input = document.getElementById('input')
const $after = document.getElementById('after')
const $number = document.getElementById('number')
const $output = document.getElementById('output')

$after.value = getNow()

$form.addEventListener('submit', e => {
  e.preventDefault()
  exec()
})

function exec () {
  const value = $input.value
  const after = $after.value
  let number = parseInt($number.value)
  if (number < 1) number = 20
  $number.value = number
  const text = cron_gate.get_datetimes(value, after, number)
  $output.value = text
}

function getNow () {
  const now = new Date()
  return `${now.getFullYear()}/${now.getMonth()}/${now.getDate()} ${now.getHours()}:${now.getMinutes()}`
}

exec()
