import { Test, TestingModule } from '@nestjs/testing';
import { ControlsService } from './controls.service';

describe('ControlsService', () => {
  let service: ControlsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ControlsService],
    }).compile();

    service = module.get<ControlsService>(ControlsService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
