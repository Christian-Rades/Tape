<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;
use Twig\Environment;
use Twig\Loader\ArrayLoader;

class VariableTest extends TestCase
{
    use SnapshotTestCase;

    private Environment $twig;

    protected function setUp(): void
    {
        $this->twig = new Environment(new ArrayLoader([]));
    }

    public function testVariableScopes()
    {
        $result = render(__DIR__ . '/fixtures/', 'variableScopes.twig', [], $this->twig);
        $this->assertSnapshot('variable_scopes', $result);
    }
}