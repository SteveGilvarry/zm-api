import { Test, TestingModule } from '@nestjs/testing';
import { ZonesService } from './zones.service';

describe('ZonesService', () => {
  let service: ZonesService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [ZonesService],
    }).compile();

    service = module.get<ZonesService>(ZonesService);
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });
});
