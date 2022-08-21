import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsUpdateInput } from './control-presets-update.input';
import { Type } from 'class-transformer';
import { ControlPresetsWhereUniqueInput } from './control-presets-where-unique.input';

@ArgsType()
export class UpdateOneControlPresetsArgs {

    @Field(() => ControlPresetsUpdateInput, {nullable:false})
    @Type(() => ControlPresetsUpdateInput)
    data!: ControlPresetsUpdateInput;

    @Field(() => ControlPresetsWhereUniqueInput, {nullable:false})
    @Type(() => ControlPresetsWhereUniqueInput)
    where!: ControlPresetsWhereUniqueInput;
}
