import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsCreateManyInput } from './zone-presets-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyZonePresetsArgs {

    @Field(() => [ZonePresetsCreateManyInput], {nullable:false})
    @Type(() => ZonePresetsCreateManyInput)
    data!: Array<ZonePresetsCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
