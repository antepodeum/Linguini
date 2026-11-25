import assert from 'node:assert/strict';
import test from 'node:test';
import {withPreparedMessages} from '../src/deferred-plan.js';

test('prepared message facts intersect all parent paths and remain sound through cycles and empty transit modules', () => {
  const boundary = (id,messages,deferred=[]) => ({id,messages,deferred});
  const cases = [
    {roots:['a'],nodes:[boundary('c',['last','shared']),boundary('b',[],['c']),boundary('a',['shared'],['b'])],
      expected:{a:['initial'],b:['initial','shared'],c:['initial','shared']}},
    {roots:['a','b'],nodes:[boundary('a',['one','shared'],['c']),boundary('b',['two','shared'],['c']),boundary('c',['last','one','two','shared'])],
      expected:{a:['initial'],b:['initial'],c:['initial','shared']}},
    {roots:['a','c'],nodes:[boundary('a',['shared'],['c']),boundary('c',['last','shared'])],
      expected:{a:['initial'],c:['initial']}},
    {roots:['a'],nodes:[boundary('b',['two'],['a']),boundary('a',['one'],['b'])],
      expected:{a:['initial'],b:['initial','one']}},
    {roots:['a','b'],nodes:[boundary('b',['two'],['a']),boundary('a',['one'],['b'])],
      expected:{a:['initial'],b:['initial']}},
  ];
  for (const {roots,nodes,expected} of cases) {
    const original = structuredClone(nodes);
    const plan = withPreparedMessages({messages:['initial'],deferred:roots},nodes);
    assert.deepEqual(Object.fromEntries(plan.map(item => [item.id,item.available])),expected);
    assert.deepEqual(nodes,original,'planning must not mutate demand records');
  }
  assert.throws(() => withPreparedMessages({messages:[],deferred:[]},[
    boundary('a',['one'],['b']),boundary('b',['two'],['a']),
  ]),/cannot determine the parent paths/);
});
