// Package mcp implements an MCP (Model Context Protocol) server that exposes
// naba's image generation capabilities as tools for AI assistants.
package mcp

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"

	mcpsdk "github.com/mark3labs/mcp-go/mcp"
	mcpserver "github.com/mark3labs/mcp-go/server"

	"github.com/dixson3/naba/internal/config"
	"github.com/dixson3/naba/internal/gemini"
	"github.com/dixson3/naba/internal/output"
)

// Serve creates and starts the MCP server on stdio.
func Serve(version string) error {
	s := mcpserver.NewMCPServer(
		"naba",
		version,
		mcpserver.WithToolCapabilities(false),
	)
	registerTools(s)
	return mcpserver.ServeStdio(s)
}

func registerTools(s *mcpserver.MCPServer) {
	s.AddTool(generateImageTool(), handleGenerateImage)
	s.AddTool(editImageTool(), handleEditImage)
	s.AddTool(restoreImageTool(), handleRestoreImage)
	s.AddTool(generateIconTool(), handleGenerateIcon)
	s.AddTool(generatePatternTool(), handleGeneratePattern)
	s.AddTool(generateStoryTool(), handleGenerateStory)
	s.AddTool(generateDiagramTool(), handleGenerateDiagram)
}

// resolveClient creates a Gemini client from config/env.
func resolveClient() (*gemini.Client, error) {
	apiKey := config.ResolveAPIKey()
	if apiKey == "" {
		return nil, fmt.Errorf("GEMINI_API_KEY not set. Set it with: export GEMINI_API_KEY=<your-key> or run: naba config set api_key <your-key>")
	}
	cfg, _ := config.Load()
	return gemini.NewClient(apiKey, cfg.Model), nil
}

// imageResult builds a CallToolResult with a text path and base64 image content.
func imageResult(path string, data []byte, mimeType string) *mcpsdk.CallToolResult {
	encoded := base64.StdEncoding.EncodeToString(data)
	return &mcpsdk.CallToolResult{
		Content: []mcpsdk.Content{
			mcpsdk.NewTextContent(path),
			mcpsdk.NewImageContent(encoded, mimeType),
		},
	}
}

// multiImageResult builds a CallToolResult from multiple images.
func multiImageResult(paths []string, images []gemini.ImageResult) *mcpsdk.CallToolResult {
	var content []mcpsdk.Content
	for i, img := range images {
		content = append(content, mcpsdk.NewTextContent(paths[i]))
		encoded := base64.StdEncoding.EncodeToString(img.Data)
		content = append(content, mcpsdk.NewImageContent(encoded, img.MIMEType))
	}
	return &mcpsdk.CallToolResult{Content: content}
}

// generateAndReturn is the common flow for text-only generation tools.
func generateAndReturn(prompt, command string) (*mcpsdk.CallToolResult, error) {
	client, err := resolveClient()
	if err != nil {
		return mcpsdk.NewToolResultError(err.Error()), nil
	}

	images, err := client.Generate(prompt)
	if err != nil {
		return mcpsdk.NewToolResultError(err.Error()), nil
	}

	if len(images) == 0 {
		return mcpsdk.NewToolResultError("no images in response"), nil
	}

	outDir := config.ResolveOutputDir()
	outPath := output.OutputPath(outDir, command, images[0].MIMEType)
	path, err := output.WriteImage(images[0].Data, images[0].MIMEType, outPath, command, 0)
	if err != nil {
		return mcpsdk.NewToolResultError(fmt.Sprintf("write image: %v", err)), nil
	}

	return imageResult(path, images[0].Data, images[0].MIMEType), nil
}

// generateWithImageAndReturn is the common flow for image+text generation tools.
func generateWithImageAndReturn(prompt, imagePath, command string) (*mcpsdk.CallToolResult, error) {
	if _, err := os.Stat(imagePath); os.IsNotExist(err) {
		return mcpsdk.NewToolResultError(fmt.Sprintf("file not found: %s", imagePath)), nil
	}

	client, err := resolveClient()
	if err != nil {
		return mcpsdk.NewToolResultError(err.Error()), nil
	}

	images, err := client.GenerateWithImage(prompt, imagePath)
	if err != nil {
		return mcpsdk.NewToolResultError(err.Error()), nil
	}

	if len(images) == 0 {
		return mcpsdk.NewToolResultError("no images in response"), nil
	}

	outDir := config.ResolveOutputDir()
	outPath := output.OutputPath(outDir, command, images[0].MIMEType)
	path, err := output.WriteImage(images[0].Data, images[0].MIMEType, outPath, command, 0)
	if err != nil {
		return mcpsdk.NewToolResultError(fmt.Sprintf("write image: %v", err)), nil
	}

	return imageResult(path, images[0].Data, images[0].MIMEType), nil
}

// handleGenerateImage handles the generate_image tool.
func handleGenerateImage(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	prompt, err := req.RequireString("prompt")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: prompt"), nil
	}

	style := req.GetString("style", "")
	variations := req.GetStringSlice("variations", nil)
	count := req.GetInt("count", 1)

	if count < 1 || count > 8 {
		return mcpsdk.NewToolResultError("count must be between 1 and 8"), nil
	}

	client, err := resolveClient()
	if err != nil {
		return mcpsdk.NewToolResultError(err.Error()), nil
	}

	enriched := gemini.EnrichGeneratePrompt(prompt, style, variations)
	outDir := config.ResolveOutputDir()
	outPath := output.OutputPath(outDir, "generate", "image/png")

	var allPaths []string
	var allImages []gemini.ImageResult

	for i := 0; i < count; i++ {
		images, err := client.Generate(enriched)
		if err != nil {
			return mcpsdk.NewToolResultError(err.Error()), nil
		}
		for j, img := range images {
			idx := i*len(images) + j
			path, err := output.WriteImage(img.Data, img.MIMEType, outPath, "generate", idx)
			if err != nil {
				return mcpsdk.NewToolResultError(fmt.Sprintf("write image: %v", err)), nil
			}
			allPaths = append(allPaths, path)
			allImages = append(allImages, img)
		}
	}

	if len(allImages) == 1 {
		return imageResult(allPaths[0], allImages[0].Data, allImages[0].MIMEType), nil
	}
	return multiImageResult(allPaths, allImages), nil
}

// handleEditImage handles the edit_image tool.
func handleEditImage(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	prompt, err := req.RequireString("prompt")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: prompt"), nil
	}
	file, err := req.RequireString("file")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: file"), nil
	}

	enriched := gemini.EnrichEditPrompt(prompt)
	return generateWithImageAndReturn(enriched, file, "edit")
}

// handleRestoreImage handles the restore_image tool.
func handleRestoreImage(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	file, err := req.RequireString("file")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: file"), nil
	}

	prompt := req.GetString("prompt", "")
	enriched := gemini.EnrichRestorePrompt(prompt)
	return generateWithImageAndReturn(enriched, file, "restore")
}

// handleGenerateIcon handles the generate_icon tool.
func handleGenerateIcon(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	prompt, err := req.RequireString("prompt")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: prompt"), nil
	}

	style := req.GetString("style", "modern")
	background := req.GetString("background", "transparent")
	corners := req.GetString("corners", "rounded")
	sizes := req.GetIntSlice("sizes", []int{256})

	client, err := resolveClient()
	if err != nil {
		return mcpsdk.NewToolResultError(err.Error()), nil
	}

	outDir := config.ResolveOutputDir()
	outPath := output.OutputPath(outDir, "icon", "image/png")

	var allPaths []string
	var allImages []gemini.ImageResult

	for i, size := range sizes {
		enriched := gemini.EnrichIconPrompt(prompt, style, size, background, corners)
		images, err := client.Generate(enriched)
		if err != nil {
			return mcpsdk.NewToolResultError(err.Error()), nil
		}
		for j, img := range images {
			idx := i*len(images) + j
			path, err := output.WriteImage(img.Data, img.MIMEType, outPath, "icon", idx)
			if err != nil {
				return mcpsdk.NewToolResultError(fmt.Sprintf("write image: %v", err)), nil
			}
			allPaths = append(allPaths, path)
			allImages = append(allImages, img)
		}
	}

	if len(allImages) == 1 {
		return imageResult(allPaths[0], allImages[0].Data, allImages[0].MIMEType), nil
	}
	return multiImageResult(allPaths, allImages), nil
}

// handleGeneratePattern handles the generate_pattern tool.
func handleGeneratePattern(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	prompt, err := req.RequireString("prompt")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: prompt"), nil
	}

	style := req.GetString("style", "abstract")
	colors := req.GetString("colors", "colorful")
	density := req.GetString("density", "medium")
	size := req.GetString("size", "256x256")
	repeat := req.GetString("repeat", "tile")

	enriched := gemini.EnrichPatternPrompt(prompt, style, colors, density, size, repeat)
	return generateAndReturn(enriched, "pattern")
}

// handleGenerateStory handles the generate_story tool.
func handleGenerateStory(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	prompt, err := req.RequireString("prompt")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: prompt"), nil
	}

	steps := req.GetInt("steps", 4)
	style := req.GetString("style", "consistent")
	transition := req.GetString("transition", "smooth")

	if steps < 2 || steps > 8 {
		return mcpsdk.NewToolResultError("steps must be between 2 and 8"), nil
	}

	client, err := resolveClient()
	if err != nil {
		return mcpsdk.NewToolResultError(err.Error()), nil
	}

	outDir := config.ResolveOutputDir()
	outPath := output.OutputPath(outDir, "story", "image/png")

	var allPaths []string
	var allImages []gemini.ImageResult

	for i := 1; i <= steps; i++ {
		enriched := gemini.EnrichStoryPrompt(prompt, i, steps, style, transition)
		images, err := client.Generate(enriched)
		if err != nil {
			return mcpsdk.NewToolResultError(err.Error()), nil
		}
		for j, img := range images {
			idx := (i-1)*len(images) + j
			path, err := output.WriteImage(img.Data, img.MIMEType, outPath, "story", idx)
			if err != nil {
				return mcpsdk.NewToolResultError(fmt.Sprintf("write image: %v", err)), nil
			}
			allPaths = append(allPaths, path)
			allImages = append(allImages, img)
		}
	}

	if len(allImages) == 1 {
		return imageResult(allPaths[0], allImages[0].Data, allImages[0].MIMEType), nil
	}
	return multiImageResult(allPaths, allImages), nil
}

// handleGenerateDiagram handles the generate_diagram tool.
func handleGenerateDiagram(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	prompt, err := req.RequireString("prompt")
	if err != nil {
		return mcpsdk.NewToolResultError("missing required parameter: prompt"), nil
	}

	diagramType := req.GetString("type", "flowchart")
	style := req.GetString("style", "professional")
	layout := req.GetString("layout", "hierarchical")
	complexity := req.GetString("complexity", "detailed")
	colors := req.GetString("colors", "accent")

	enriched := gemini.EnrichDiagramPrompt(prompt, diagramType, style, layout, complexity, colors)
	return generateAndReturn(enriched, "diagram")
}
