<?php

namespace Tests;

use PHPUnit\Framework\TestCase;

class SmokeTest extends TestCase
{
    public function testBasicTest()
    {
        $result = render(__DIR__ . '/fixtures/', 'basic.html.twig', ['foo' => ['name' => 'John'], 'coll' => ['a', 'b', 'c']]);
        $this->assertEquals('hello world', $result);
    }

    public function testBenchmark() {
        $start = microtime(true);
        for ($i = 0; $i < 1000; $i++) {
            $result = render(__DIR__ . '/fixtures/', 'basic.html.twig', ['foo' => ['name' => 'John'], 'coll' => ['a', 'b', 'c']]);
        }
        $end = microtime(true);
        $this->assertLessThan(0.0, $end - $start);
    }
}