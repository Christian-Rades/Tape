<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;
use Twig\Environment;
use Twig\Loader\ArrayLoader;

class IncludesTest extends TestCase
{
    use SnapshotTestCase;

    private Environment $twig;

    protected function setUp(): void
    {
        $this->twig = new Environment(new ArrayLoader([]));
    }

    public function testInclude()
    {
        $result = render(__DIR__ . '/fixtures/', 'include.twig', [], $this->twig);
        $this->assertSnapshot('include', $result);
    }
}