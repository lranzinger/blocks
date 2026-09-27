let pageHidden = false;
document.addEventListener('visibilitychange', function () {
  if (document.hidden) {
    pageHidden = true;
  }
});

// Exact time of every touch change. The game only runs once per frame, with a low frame
// rate the frame time would make taps and flicks look longer than they were.
const touchTimes = [];
function recordTouches(event) {
  for (const touch of event.changedTouches) {
    touchTimes.push([touch.identifier >>> 0, event.timeStamp / 1000]);
  }
}
for (const type of ['touchstart', 'touchmove', 'touchend', 'touchcancel']) {
  canvas.addEventListener(type, recordTouches, { passive: true });
}

let platform = function (importObject) {
  importObject.env.js_get_high_score = function () {
    const score = localStorage.getItem('blocks_high_score') || '0';
    return parseInt(score);
  };

  importObject.env.js_save_high_score = function (score) {
    localStorage.setItem('blocks_high_score', score.toString());
  };

  // Devices mainly used with a finger, like phones and tablets. iPads count as well,
  // although Safari reports them as Mac.
  importObject.env.js_is_touch_device = function () {
    const coarse = window.matchMedia('(pointer: coarse)').matches;
    const touchOnly = navigator.maxTouchPoints > 1 && !window.matchMedia('(any-pointer: fine)').matches;
    return coarse || touchOnly ? 1 : 0;
  };

  importObject.env.js_take_page_hidden = function () {
    const hidden = pageHidden;
    pageHidden = false;
    return hidden ? 1 : 0;
  };

  // Same clock as the touch times, in seconds
  importObject.env.js_now = function () {
    return performance.now() / 1000;
  };

  // Time of the next touch change in the order the game receives them, -1 if unknown
  importObject.env.js_next_touch_time = function (identifier) {
    const next = touchTimes.shift();
    if (next === undefined || next[0] !== identifier) {
      touchTimes.length = 0;
      return -1;
    }
    return next[1];
  };

  importObject.env.js_clear_touch_times = function () {
    touchTimes.length = 0;
  };

  importObject.env.js_vibrate = function (milliseconds) {
    if (navigator.vibrate) {
      navigator.vibrate(milliseconds);
    }
  };
};
miniquad_add_plugin({ register_plugin: platform, version: 1, name: "platform" });
