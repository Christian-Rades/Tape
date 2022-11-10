<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;
use Twig\Environment;
use Twig\Loader\ArrayLoader;

class SmokeTest extends TestCase
{
    use SnapshotTestCase;

    private Environment $twig;

    protected function setUp(): void
    {
        $this->twig = new Environment(new ArrayLoader([]));
    }

    public function testBasicTest()
    {
        $result = render(__DIR__ . '/fixtures/', 'basic.html.twig', ['foo' => ['name' => 'John'], 'coll' => ['a', 'b', 'c']], $this->twig);
        $this->assertSnapshot('basic', $result);
    }
}