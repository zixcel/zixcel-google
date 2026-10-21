import assert from 'node:assert/strict'
import test from 'node:test'
import { describeService, createNativePlanner } from '../dist/index.mjs'
import { selection } from './native-support.mts'
const planner = createNativePlanner(selection), serviceProfile = describeService(planner), operations = planner.catalog.operations
test('public service profile exposes declarations without callbacks or claimed observed permissions', () => {
  assert.equal(serviceProfile.permissionState, 'not-observed')
  assert.equal(serviceProfile.operations.length, operations.length)
  assert.ok(Object.isFrozen(serviceProfile.operations))
  for (const operation of serviceProfile.operations) {
    assert.ok(Object.isFrozen(operation.required_permissions))
    assert.equal(operation.build, undefined)
  }
})
