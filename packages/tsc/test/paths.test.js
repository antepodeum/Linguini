import assert from 'node:assert/strict';
import path from 'node:path';
import test from 'node:test';
import {isWithin} from '../src/paths.js';

test('source membership follows POSIX and Windows components, separators, drives, and case rules', () => {
  const cases = [
    [path.posix,'/app/schema','/app/schema',true],
    [path.posix,'/app/schema','/app/schema/new.lgs',true],
    [path.posix,'/app/schema','/app//schema/new.lgs',true],
    [path.posix,'/app/schema','/app/schema/nested/../new.lgs',true],
    [path.posix,'/app/schema','/app/Schema/new.lgs',false],
    [path.posix,'/app/schema','/app/schema-sibling/new.lgs',false],
    [path.posix,'/app/schema','/app/schema/../locale/new.lgl',false],
    [path.win32,'D:\\App\\schema','d:/app/schema/new.lgs',true],
    [path.win32,'D:/App/schema','D:\\App\\schema\\new.lgs',true],
    [path.win32,'D:\\App\\schema','D:/App//schema/new.lgs',true],
    [path.win32,'D:\\App\\schema','D:/App/schema/nested/../new.lgs',true],
    [path.win32,'D:\\App\\schema','D:/App/schema-sibling/new.lgs',false],
    [path.win32,'D:\\App\\schema','D:/App/schema/../locale/new.lgl',false],
    [path.win32,'D:\\App\\schema','E:/App/schema/new.lgs',false],
    [path.win32,'\\\\server\\share\\App\\schema','//SERVER/share/App/schema/new.lgs',true],
    [path.win32,'\\\\server\\share\\App\\schema','//server/other/App/schema/new.lgs',false],
  ];
  for (const [paths,root,file,expected] of cases) assert.equal(isWithin(root,file,paths),expected,`${root} -> ${file}`);
});
