<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;

class SmokeTest extends TestCase
{
    use SnapshotTestCase;

    public function testBasicTest()
    {
        $result = render(__DIR__ . '/fixtures/', 'basic.html.twig', ['foo' => ['name' => 'John'], 'coll' => ['a', 'b', 'c']]);
        $this->assertSnapshot('basic', $result);
    }
}