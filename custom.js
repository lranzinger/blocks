let storage = function (importObject) {
  importObject.env.js_get_high_score = function () {
    const score = localStorage.getItem('blocks_high_score') || '0';
    return parseInt(score);
  };

  importObject.env.js_save_high_score = function (score) {
    localStorage.setItem('blocks_high_score', score.toString());
  };
};
miniquad_add_plugin({ register_plugin: storage, version: 1, name: "storage" });

