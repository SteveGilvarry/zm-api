import { Test, TestingModule } from '@nestjs/testing';
import { FramesService } from './frames.service';

describe('FramesService', () => {
  let service: FramesService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [FramesService],
    }).compile();

    service = module.get<FramesService>(FramesService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
