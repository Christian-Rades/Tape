<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;

class IncludesTest extends TestCase
{
    use SnapshotTestCase;

    public function testInclude()
    {
        $result = render(__DIR__ . '/fixtures/', 'include.twig', []);
        $this->assertSnapshot('include', $result);
    }
}