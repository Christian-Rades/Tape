<?php

namespace Test;

use PHPUnit\Framework\TestCase;
use Test\Utils\SnapshotTestCase;

class VariableTest extends TestCase
{
    use SnapshotTestCase;

    public function testVariableScopes()
    {
        $result = render(__DIR__ . '/fixtures/', 'variableScopes.twig', []);
        $this->assertSnapshot('variable_scopes', $result);
    }
}