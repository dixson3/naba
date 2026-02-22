// Package mcp implements an MCP (Model Context Protocol) server that exposes
// naba's image generation capabilities as tools for AI assistants.
package mcp

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"

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
		mcpserver.WithResourceCapabilities(false, false),
	)
	registerTools(s)
	registerResources(s)
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
	s.AddTool(listImagesTool(), handleListImages)
}

func registerResources(s *mcpserver.MCPServer) {
	s.AddResourceTemplate(
		mcpsdk.NewResourceTemplate(
			"file:///{path}",
			"Generated image file",
			mcpsdk.WithTemplateDescription("Access a generated image by its file path"),
			mcpsdk.WithTemplateMIMEType("image/*"),
		),
		handleReadResource,
	)
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

// resolveOutputDirWithDefault returns the configured output directory,
// falling back to the XDG default (~/.local/share/naba/images) when unset.
func resolveOutputDirWithDefault() string {
	if dir := config.ResolveOutputDir(); dir != "" {
		return dir
	}
	return config.DefaultOutputDir()
}

// imageResult builds a CallToolResult with a text path and resource link.
func imageResult(path string, _ []byte, mimeType string) *mcpsdk.CallToolResult {
	return &mcpsdk.CallToolResult{
		Content: []mcpsdk.Content{
			mcpsdk.NewTextContent(path),
			mcpsdk.NewResourceLink(
				"file://"+path,
				filepath.Base(path),
				"Generated image",
				mimeType,
			),
		},
	}
}

// multiImageResult builds a CallToolResult from multiple images with resource links.
func multiImageResult(paths []string, images []gemini.ImageResult) *mcpsdk.CallToolResult {
	var content []mcpsdk.Content
	for i, img := range images {
		content = append(content,
			mcpsdk.NewTextContent(paths[i]),
			mcpsdk.NewResourceLink(
				"file://"+paths[i],
				filepath.Base(paths[i]),
				fmt.Sprintf("Generated image %d", i+1),
				img.MIMEType,
			),
		)
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

	outDir := resolveOutputDirWithDefault()
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

	outDir := resolveOutputDirWithDefault()
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
	outDir := resolveOutputDirWithDefault()
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

	outDir := resolveOutputDirWithDefault()
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

	outDir := resolveOutputDirWithDefault()
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

// handleListImages lists recently generated images in the output directory.
func handleListImages(_ context.Context, req mcpsdk.CallToolRequest) (*mcpsdk.CallToolResult, error) {
	outDir := resolveOutputDirWithDefault()
	if outDir == "" {
		return mcpsdk.NewToolResultError("no output directory configured"), nil
	}

	limit := req.GetInt("limit", 20)
	if limit < 1 {
		limit = 20
	}

	entries, err := os.ReadDir(outDir)
	if err != nil {
		if os.IsNotExist(err) {
			return &mcpsdk.CallToolResult{
				Content: []mcpsdk.Content{
					mcpsdk.NewTextContent("No images found (directory does not exist)"),
				},
			}, nil
		}
		return mcpsdk.NewToolResultError(fmt.Sprintf("read output directory: %v", err)), nil
	}

	// Filter to naba-* image files and collect with mod times
	type fileEntry struct {
		path    string
		modTime int64
	}
	var files []fileEntry
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		name := e.Name()
		if !strings.HasPrefix(name, "naba-") {
			continue
		}
		ext := strings.ToLower(filepath.Ext(name))
		if ext != ".png" && ext != ".jpg" && ext != ".jpeg" && ext != ".gif" && ext != ".webp" {
			continue
		}
		info, err := e.Info()
		if err != nil {
			continue
		}
		files = append(files, fileEntry{
			path:    filepath.Join(outDir, name),
			modTime: info.ModTime().UnixNano(),
		})
	}

	// Sort newest first
	sort.Slice(files, func(i, j int) bool {
		return files[i].modTime > files[j].modTime
	})

	if len(files) > limit {
		files = files[:limit]
	}

	if len(files) == 0 {
		return &mcpsdk.CallToolResult{
			Content: []mcpsdk.Content{
				mcpsdk.NewTextContent("No images found"),
			},
		}, nil
	}

	var content []mcpsdk.Content
	for _, f := range files {
		content = append(content, mcpsdk.NewTextContent(f.path))
	}
	return &mcpsdk.CallToolResult{Content: content}, nil
}

// handleReadResource reads a generated image file by its file:// URI.
func handleReadResource(_ context.Context, req mcpsdk.ReadResourceRequest) ([]mcpsdk.ResourceContents, error) {
	path := strings.TrimPrefix(req.Params.URI, "file://")
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read image: %w", err)
	}
	mime := mimeFromExt(filepath.Ext(path))
	encoded := base64.StdEncoding.EncodeToString(data)
	return []mcpsdk.ResourceContents{
		mcpsdk.BlobResourceContents{
			URI:      req.Params.URI,
			MIMEType: mime,
			Blob:     encoded,
		},
	}, nil
}

// mimeFromExt returns the MIME type for a file extension.
func mimeFromExt(ext string) string {
	switch strings.ToLower(ext) {
	case ".png":
		return "image/png"
	case ".jpg", ".jpeg":
		return "image/jpeg"
	case ".gif":
		return "image/gif"
	case ".webp":
		return "image/webp"
	default:
		return "application/octet-stream"
	}
}
