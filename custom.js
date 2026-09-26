let storage = function (importObject) {
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

  importObject.env.js_vibrate = function (milliseconds) {
    if (navigator.vibrate) {
      navigator.vibrate(milliseconds);
    }
  };
};
miniquad_add_plugin({ register_plugin: storage, version: 1, name: "storage" });
