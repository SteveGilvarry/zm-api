import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ZonePresetsWhereInput } from './zone-presets-where.input';

@ArgsType()
export class DeleteManyZonePresetsArgs {

    @Field(() => ZonePresetsWhereInput, {nullable:true})
    where?: ZonePresetsWhereInput;
}
