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
}