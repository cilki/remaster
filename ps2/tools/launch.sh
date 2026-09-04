#!/usr/bin/env bash
# launch.sh <iso> [wait_s] — cleanly (re)launch PCSX2 under cage and wait.
set -u
iso="$1"; wait_s="${2:-25}"
pkill -x .pcsx2-qt-wrapp 2>/dev/null; pkill -x cage 2>/dev/null
for i in $(seq 1 20); do
  pgrep -x .pcsx2-qt-wrapp >/dev/null || pgrep -x cage >/dev/null || break
  sleep 0.5
done
rm -f /tmp/cagert/wayland-0 /tmp/cagert/wayland-0.lock /dev/shm/pcsx2*
cage -- pcsx2-qt -batch -fullscreen "$iso" > /tmp/cage.log 2>&1 &
sleep "$wait_s"
if ! pgrep -x .pcsx2-qt-wrapp >/dev/null; then
  echo "LAUNCH FAILED:"; tail -5 /tmp/cage.log; exit 1
fi
echo "pcsx2 running"
