// Set bundle to evaluation mode
// Reference: https://github.com/remotion-dev/remotion/blob/main/packages/renderer/src/get-compositions.ts#L104C4-L106C7
window.remotion_setBundleMode({ type: 'evaluation' });

// Get a list of compositions, 
// Reference: https://github.com/remotion-dev/remotion/blob/main/packages/renderer/src/get-compositions.ts#L114C3-L117C15
window.getStaticCompositions();

// Result of window.getStaticCompositions();
[
	{
		"width": 1920,
		"height": 1080,
		"fps": 30,
		"durationInFrames": 150,
		"id": "HelloWorld",
		"serializedResolvedPropsWithCustomSchema": "{\"titleText\":\"Welcome to Remotion\",\"titleColor\":\"#000000\",\"logoColor1\":\"#91EAE4\",\"logoColor2\":\"#86A8E7\"}",
		"serializedDefaultPropsWithCustomSchema": "{\"titleText\":\"Welcome to Remotion\",\"titleColor\":\"#000000\",\"logoColor1\":\"#91EAE4\",\"logoColor2\":\"#86A8E7\"}"
	},
	{
		"width": 1920,
		"height": 1080,
		"fps": 30,
		"durationInFrames": 150,
		"id": "OnlyLogo",
		"serializedResolvedPropsWithCustomSchema": "{\"logoColor1\":\"#91dAE2\",\"logoColor2\":\"#86A8E7\"}",
		"serializedDefaultPropsWithCustomSchema": "{\"logoColor1\":\"#91dAE2\",\"logoColor2\":\"#86A8E7\"}"
	}
]

// Set bundle to composition mode — props need to be correctly provided.
// Reference: https://github.com/remotion-dev/remotion/blob/main/packages/renderer/src/render-frames.ts#L283C5-L291C8
window.remotion_setBundleMode({
	type: 'composition',
	compositionName: id,
	serializedResolvedPropsWithSchema: props,
	compositionDurationInFrames: durationInFrames,
	compositionFps: fps,
	compositionHeight: height,
	compositionWidth: width,
});


// In composition mode, "seek" to frame (f: frame number, c: composition name)
// Reference: https://github.com/remotion-dev/remotion/blob/main/packages/renderer/src/seek-to-frame.ts#L159C3-L162C30
window.remotion_setFrame(f, c);





