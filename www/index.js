import * as cron_gate from '../pkg/cron_gate'

const $form = document.getElementById('form')
const $input = document.getElementById('input')
const $after = document.getElementById('after')
const $output = document.getElementById('output')

$after.value = getNow()

$form.addEventListener('submit', e => {
  e.preventDefault()
  exec()
})

function exec () {
  const value = $input.value
  const after = $after.value
  const text = cron_gate.get_datetimes(value, after, 20)
  $output.value = text
  console.log(text)
}

function getNow () {
  const now = new Date()
  return `${now.getFullYear()}/${now.getMonth()}/${now.getDate()} ${now.getHours()}:${now.getMinutes()}`
}

exec()
