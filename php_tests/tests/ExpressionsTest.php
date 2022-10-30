<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;

class ExpressionsTest extends TestCase
{
    use SnapshotTestCase;


    public function testArithmetic()
    {
        $result = render(__DIR__ . '/fixtures/', 'arithmetic.twig', []);
        $this->assertSnapshot('arithmetic', $result);
    }

    public function testLogic()
    {
        $result = render(__DIR__ . '/fixtures/', 'logic.twig', []);
        $this->assertSnapshot('logic', $result);
    }

    public function testStringConcat()
    {
        $result = render(__DIR__ . '/fixtures/', 'strConcat.twig', []);
        $this->assertSnapshot('strConcat', $result);
    }
}
