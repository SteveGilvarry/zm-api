import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereUniqueInput } from './zone-presets-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneZonePresetsArgs {

    @Field(() => ZonePresetsWhereUniqueInput, {nullable:false})
    @Type(() => ZonePresetsWhereUniqueInput)
    where!: ZonePresetsWhereUniqueInput;
}
