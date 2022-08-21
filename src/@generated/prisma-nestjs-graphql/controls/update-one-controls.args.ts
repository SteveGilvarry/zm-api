import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsUpdateInput } from './controls-update.input';
import { Type } from 'class-transformer';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';

@ArgsType()
export class UpdateOneControlsArgs {

    @Field(() => ControlsUpdateInput, {nullable:false})
    @Type(() => ControlsUpdateInput)
    data!: ControlsUpdateInput;

    @Field(() => ControlsWhereUniqueInput, {nullable:false})
    @Type(() => ControlsWhereUniqueInput)
    where!: ControlsWhereUniqueInput;
}
