"use strict";
(self["webpackChunkremotion_template_three"] = self["webpackChunkremotion_template_three"] || []).push([[618],{

/***/ 618:
/***/ ((__unused_webpack___webpack_module__, __webpack_exports__, __webpack_require__) => {

__webpack_require__.r(__webpack_exports__);
/* harmony export */ __webpack_require__.d(__webpack_exports__, {
/* harmony export */   VideoTexture: () => (/* binding */ VideoTexture)
/* harmony export */ });
/* harmony import */ var _constants_js__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(7006);
/* harmony import */ var _Texture_js__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(8603);



class VideoTexture extends _Texture_js__WEBPACK_IMPORTED_MODULE_0__/* .Texture */ .x {

	constructor( video, mapping, wrapS, wrapT, magFilter, minFilter, format, type, anisotropy ) {

		super( video, mapping, wrapS, wrapT, magFilter, minFilter, format, type, anisotropy );

		this.isVideoTexture = true;

		this.minFilter = minFilter !== undefined ? minFilter : _constants_js__WEBPACK_IMPORTED_MODULE_1__/* .LinearFilter */ .we;
		this.magFilter = magFilter !== undefined ? magFilter : _constants_js__WEBPACK_IMPORTED_MODULE_1__/* .LinearFilter */ .we;

		this.generateMipmaps = false;

		const scope = this;

		function updateVideo() {

			scope.needsUpdate = true;
			video.requestVideoFrameCallback( updateVideo );

		}

		if ( 'requestVideoFrameCallback' in video ) {

			video.requestVideoFrameCallback( updateVideo );

		}

	}

	clone() {

		return new this.constructor( this.image ).copy( this );

	}

	update() {

		const video = this.image;
		const hasVideoFrameCallback = 'requestVideoFrameCallback' in video;

		if ( hasVideoFrameCallback === false && video.readyState >= video.HAVE_CURRENT_DATA ) {

			this.needsUpdate = true;

		}

	}

}




/***/ })

}]);
//# sourceMappingURL=618.bundle.js.map