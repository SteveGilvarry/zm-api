import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlsUpdateInput } from './controls-update.input';
import { ControlsWhereUniqueInput } from './controls-where-unique.input';

@ArgsType()
export class UpdateOneControlsArgs {

    @Field(() => ControlsUpdateInput, {nullable:false})
    data!: ControlsUpdateInput;

    @Field(() => ControlsWhereUniqueInput, {nullable:false})
    where!: ControlsWhereUniqueInput;
}
