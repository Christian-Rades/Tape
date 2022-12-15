<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;
use Twig\Environment;
use Twig\Loader\ArrayLoader;

class ExpressionsTest extends TestCase
{
    use SnapshotTestCase;

    private Environment $twig;

    protected function setUp(): void
    {
        $this->twig = new Environment(new ArrayLoader([]));
    }

    public function testArithmetic()
    {
        $result = render(__DIR__ . '/fixtures/', 'arithmetic.twig', [], $this->twig);
        $this->assertSnapshot('arithmetic', $result);
    }

    public function testLogic()
    {
        $result = render(__DIR__ . '/fixtures/', 'logic.twig', [], $this->twig);
        $this->assertSnapshot('logic', $result);
    }

    public function testStringConcat()
    {
        $result = render(__DIR__ . '/fixtures/', 'strConcat.twig', [], $this->twig);
        $this->assertSnapshot('strConcat', $result);
    }

    public function testFunctionCall()
    {
        $result = render(__DIR__ . '/fixtures/', 'func.twig', [], $this->twig);
        $this->assertSnapshot('func', $result);
    }

    public function testFilter()
    {
        $result = render(__DIR__ . '/fixtures/', 'filter.twig', ['d' => new \DateTimeImmutable('2000-01-01')], $this->twig);
        $this->assertSnapshot('filter', $result);
    }

    public function testArrayLiteral()
    {
        $result = render(__DIR__ . '/fixtures/', 'array.twig', [], $this->twig);
        $this->assertSnapshot('array', $result);
    }

    public function testHashmapLiteral()
    {
        $result = render(__DIR__ . '/fixtures/', 'assocArray.twig', [], $this->twig);
        $this->assertSnapshot('assocArray', $result);
    }
}
