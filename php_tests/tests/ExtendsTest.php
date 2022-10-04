<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;

class ExtendsTest extends TestCase
{
    use SnapshotTestCase;

    public function testBasicExtends()
    {
        $result = render(__DIR__ . '/fixtures/', 'extension.twig', []);
        $this->assertSnapshot('extends_basic', $result);
    }
}