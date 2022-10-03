<?php

namespace Test\Utils;

trait SnapshotTestCase
{
    public function assertSnapshot(string $snapshotName, string $actual)
    {
        $snapshotFile = $this->getSnapshotFile($snapshotName);
        if (!file_exists($snapshotFile) || ($_ENV['UPDATE_SNAPSHOTS'] ?? false)) {
            $this->createSnapshot($snapshotFile, $actual);
        }
        static::assertEquals(file_get_contents($snapshotFile), $actual);
    }

    private function getSnapshotName()
    {
        $trace = debug_backtrace(DEBUG_BACKTRACE_IGNORE_ARGS, 3);
        $testClass = $trace[1]['class'];
        $testMethod = $trace[1]['function'];
        $snapshotName = $testClass . '::' . $testMethod;
        return $snapshotName;
    }

    private function getSnapshotFile($snapshotName)
    {
        $snapshotFile = __DIR__ . '/../fixtures/snapshots/' . $snapshotName . '.txt';
        return $snapshotFile;
    }

    private function createSnapshot($snapshotFile, $actual)
    {
        $dir = dirname($snapshotFile);
        if (!file_exists($dir)) {
            mkdir($dir, 0777, true);
        }
        file_put_contents($snapshotFile, $actual);
    }
}