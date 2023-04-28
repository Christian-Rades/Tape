<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;
use Twig\Environment;
use Twig\Loader\ArrayLoader;

class IfBlockTest extends TestCase
{
    use SnapshotTestCase;

    private Environment $twig;

    protected function setUp(): void
    {
        $this->twig = new Environment(new ArrayLoader([]));
    }

    public function testif()
    {
        $result = render(__DIR__ . '/fixtures/', 'if.twig', [], $this->twig);
        $this->assertSnapshot('if', $result);
    }
}
