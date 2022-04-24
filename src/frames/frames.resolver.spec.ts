import { Test, TestingModule } from '@nestjs/testing';
import { FramesResolver } from './frames.resolver';
import { FramesService } from './frames.service';

describe('FramesResolver', () => {
  let resolver: FramesResolver;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [FramesResolver, FramesService],
    }).compile();

    resolver = module.get<FramesResolver>(FramesResolver);
  });

  it('should be defined', () => {
    expect(resolver).toBeDefined();
  });
});
